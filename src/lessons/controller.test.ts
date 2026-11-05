import { describe, expect, it, vi } from 'vitest';
import { LessonController } from './controller';
import type { Composition, LessonDraft, LessonPort } from './types';

export const composition = (unitId = 'english'): Composition => ({ contractVersion: 1, unitId, rootNodeId: 'root', nodes: [{ nodeId: 'root', typeId: 'stack', typeVersion: 1, props: { gap: 'normal', headingLevel: 2 }, slots: { body: [] }, on: {} }], bindings: [], sourceBindingRefs: [], appearance: { spacing: 'normal', theme: 'system', typography: 'default', width: 'normal', emphasis: 'normal' }, authoringProvenance: { kind: 'manual' } });
function port() {
  const draft: LessonDraft = { id: 'draft', unitId: 'english', revision: 1, composition: composition(), baseHeadRevision: null };
  return { discover: vi.fn(async () => []), read: vi.fn(async () => null), listDrafts: vi.fn(async () => [structuredClone(draft)]), editDraft: vi.fn(async (r) => ({ ...draft, revision: r.expectedDraftRevision + 1 })), validateDraft: vi.fn(async () => ({ valid: true, diagnostics: [] })), previewDraft: vi.fn(async () => ({ draftId: 'draft', draftRevision: 1, composition: draft.composition, hash: 'exact-hash', warnings: [] })), applyDraft: vi.fn(), getReceipt: vi.fn(async () => null), readActivity: vi.fn(async () => null), readResource: vi.fn(async () => null), readSource: vi.fn(async () => null) } as unknown as LessonPort;
}
describe('durable lesson controller', () => {
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
    const receipt = { operationId: 'receipt', saved, newHead: 'saved', committed: true as const };
    vi.mocked(p.applyDraft).mockRejectedValueOnce(new Error('lost')); vi.mocked(p.getReceipt).mockResolvedValue(receipt as never);
    const c = new LessonController('workspace', 'english', p); await c.load(); await c.save();
    const request = vi.mocked(p.applyDraft).mock.calls[0][0]; expect(request.previewHash).toBe('exact-hash');
    await c.retry(); expect(p.getReceipt).toHaveBeenCalledWith(request.operationId);
    expect(c.getSnapshot().saved?.id).toBe('saved'); expect(c.getSnapshot().draft).toBeNull();
  });
});
