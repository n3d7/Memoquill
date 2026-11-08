import { describe, expect, it, vi } from 'vitest';
import { LessonController, restoreOperations } from './controller';
import type { Composition, LessonDraft, LessonPort } from './types';

export const composition = (unitId = 'english'): Composition => ({ contractVersion: 1, unitId, rootNodeId: 'root', nodes: [{ nodeId: 'root', typeId: 'stack', typeVersion: 1, props: { gap: 'normal', headingLevel: 2 }, slots: { body: [] }, on: {} }], bindings: [], sourceBindingRefs: [], appearance: { spacing: 'normal', theme: 'system', typography: 'default', width: 'normal', emphasis: 'normal' }, authoringProvenance: { kind: 'manual' } });
function port() {
  const draft: LessonDraft = { id: 'draft', unitId: 'english', revision: 1, composition: composition(), baseHeadRevision: null };
  return { discover: vi.fn(async () => []), read: vi.fn(async () => null), listDrafts: vi.fn(async () => [structuredClone(draft)]), editDraft: vi.fn(async (r) => ({ ...draft, revision: r.expectedDraftRevision + 1 })), validateDraft: vi.fn(async () => ({ valid: true, diagnostics: [] })), previewDraft: vi.fn(async () => ({ draftId: 'draft', draftRevision: 1, composition: draft.composition, hash: 'exact-hash', warnings: [] })), applyDraft: vi.fn(), getReceipt: vi.fn(async () => null), readActivity: vi.fn(async () => null), readResource: vi.fn(async () => null), readSource: vi.fn(async () => null) } as unknown as LessonPort;
}
describe('durable lesson controller', () => {
  it('keeps form editing available after a definitive rejected change', async () => {
    const p = port(); vi.mocked(p.editDraft).mockRejectedValueOnce({ code: 'invalid_request' }); const c = new LessonController('workspace', 'english', p); await c.load();
    await c.edit([{ kind: 'configureNode', id: 'root', props: { gap: 'invalid' } }]); expect(c.getSnapshot().pending).toBe(false); expect(c.getSnapshot().draft?.revision).toBe(1);
    await c.edit([{ kind: 'configureNode', id: 'root', props: { gap: 'normal', headingLevel: 2 } }]); expect(p.editDraft).toHaveBeenCalledTimes(2);
  });
  it('restores a removed large subtree with one semantic operation and stable node IDs', () => {
    const target = composition(); target.nodes[0].slots.body = ['columns']; target.nodes.push({ nodeId: 'columns', typeId: 'columns', typeVersion: 1, props: { ratio: 'equal' }, slots: { primary: [], secondary: [] }, on: {} });
    for (let i = 0; i < 70; i++) { const nodeId = `node-${i}`; target.nodes[1].slots[i < 35 ? 'primary' : 'secondary'].push(nodeId); target.nodes.push({ nodeId, typeId: 'status', typeVersion: 1, props: { field: 'response.present', label: 'Response' }, slots: {}, on: {} }); }
    const operations = restoreOperations(composition(), target); expect(operations).toHaveLength(1); expect(operations[0]).toMatchObject({ kind: 'insertSubtree', rootNodeId: 'columns', nodes: target.nodes.slice(1) });
  });
  it('never applies a previous preview when fresh validation fails', async () => {
    const p = port(); const c = new LessonController('workspace', 'english', p); await c.load(); await c.preview();
    vi.mocked(p.validateDraft).mockResolvedValueOnce({ draftId: 'draft', draftRevision: 1, valid: false, diagnostics: [{ code: 'invalid_tree', field: 'nodes', repair: 'correct_tree' }] }); await c.save(); expect(p.applyDraft).not.toHaveBeenCalled();
  });
  it('exposes explicit base recovery when validation itself reports a conflict', async () => {
    const p = port(); vi.mocked(p.validateDraft).mockRejectedValueOnce({ code: 'revision_conflict' }); const c = new LessonController('workspace', 'english', p); await c.load(); await c.preview(); expect(c.getSnapshot().conflict).toBe(true); expect(c.getSnapshot().pending).toBe(false);
  });
  it('undoes a saved appearance change through a new draft without erasing the saved revision', async () => {
    const p = port(); const dark = composition(); dark.appearance.theme = 'dark';
    vi.mocked(p.editDraft).mockResolvedValueOnce({ id: 'draft', unitId: 'english', revision: 2, composition: dark, baseHeadRevision: null }).mockResolvedValueOnce({ id: 'fresh', unitId: 'english', revision: 2, composition: composition(), baseHeadRevision: 'saved' });
    vi.mocked(p.previewDraft).mockResolvedValue({ draftId: 'draft', draftRevision: 2, composition: dark, hash: 'dark-hash', warnings: [] });
    vi.mocked(p.applyDraft).mockImplementation(async request => ({ operationId: request.operationId, proposalHash: request.previewHash, committed: true, saved: { id: 'saved', unitId: 'english', composition: dark }, newHead: 'saved' } as never));
    p.createDraft = vi.fn(async request => ({ id: 'fresh', unitId: 'english', revision: 1, composition: dark, baseHeadRevision: request.expectedHeadRevision }));
    const c = new LessonController('workspace', 'english', p); await c.load(); await c.edit([{ kind: 'setAppearance', tokens: dark.appearance }]); await c.save(); expect(c.getSnapshot().canUndo).toBe(true); await c.undo();
    expect(p.createDraft).toHaveBeenCalledWith(expect.objectContaining({ expectedHeadRevision: 'saved' })); expect(c.getSnapshot().saved?.id).toBe('saved'); expect(c.getSnapshot().composition?.appearance.theme).toBe('system'); expect(c.getSnapshot().canRedo).toBe(true);
  });
  it('offers explicit fresh-base recovery after a conflict and preserves the previous durable draft', async () => {
    const p = port(); vi.mocked(p.editDraft).mockRejectedValueOnce({ code: 'revision_conflict' });
    p.createDraft = vi.fn(async request => ({ id: 'fresh-draft', unitId: 'english', revision: 1, composition: composition(), baseHeadRevision: request.expectedHeadRevision }));
    const c = new LessonController('workspace', 'english', p); await c.load(); await c.edit([{ kind: 'setAppearance', tokens: composition().appearance }]);
    vi.mocked(p.read).mockResolvedValue({ id: 'new-head', unitId: 'english', composition: composition(), parentRevisionId: null, registryContractVersion: 1, hash: 'hash' }); await c.startLatestDraft();
    expect(p.createDraft).toHaveBeenCalledWith(expect.objectContaining({ expectedHeadRevision: 'new-head' })); expect(c.getSnapshot().draft?.id).toBe('fresh-draft'); expect(c.getSnapshot().drafts.map(d => d.id)).toContain('draft');
  });
  it('resumes an existing durable draft without replacing it', async () => {
    const p = port(); const c = new LessonController('workspace', 'english', p); await c.load();
    expect(c.getSnapshot().draft?.id).toBe('draft');
    expect(c.getSnapshot().composition?.unitId).toBe('english');
  });
  it('retries the exact edit request after a lost reply and blocks conflicting edits', async () => {
    const p = port(); vi.mocked(p.editDraft).mockRejectedValueOnce(new Error('reply lost'));
    const c = new LessonController('workspace', 'english', p); await c.load();
    await c.edit([{ kind: 'setAppearance', tokens: { ...composition().appearance, theme: 'dark' } }]);
    const first = vi.mocked(p.editDraft).mock.calls[0][0];
    await c.edit([{ kind: 'setAppearance', tokens: composition().appearance }]);
    expect(p.editDraft).toHaveBeenCalledTimes(1);
    await c.retry(); expect(vi.mocked(p.editDraft).mock.calls[1][0]).toEqual(first);
    expect(c.getSnapshot().pending).toBe(false);
  });
  it('saves the exact validated preview and recovers its receipt after a lost acknowledgement', async () => {
    const p = port(); const saved = { id: 'saved', unitId: 'english', composition: composition(), parentRevisionId: null, registryContractVersion: 1 as const, hash: 'exact-hash' };
    vi.mocked(p.applyDraft).mockRejectedValueOnce(new Error('lost')); vi.mocked(p.getReceipt).mockImplementation(async operationId => ({ operationId, saved, proposalHash: 'exact-hash', newHead: 'saved', committed: true as const } as never));
    const c = new LessonController('workspace', 'english', p); await c.load(); await c.save();
    const request = vi.mocked(p.applyDraft).mock.calls[0][0]; expect(request.previewHash).toBe('exact-hash');
    await c.retry(); expect(p.getReceipt).toHaveBeenCalledWith(request.operationId);
    expect(c.getSnapshot().saved?.id).toBe('saved'); expect(c.getSnapshot().draft).toBeNull();
  });
  it('refuses a mismatched receipt rather than claiming another request was saved', async () => {
    const p = port(); vi.mocked(p.applyDraft).mockResolvedValue({ operationId: 'different-request', proposalHash: 'exact-hash', committed: true, saved: { id: 'wrong', unitId: 'english', composition: composition() } } as never);
    const c = new LessonController('workspace', 'english', p); await c.load(); await c.save(); expect(c.getSnapshot().saved).toBeNull(); expect(c.getSnapshot().pending).toBe(true); expect(c.getSnapshot().draft?.id).toBe('draft');
  });
});
