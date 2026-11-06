import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { LessonComposer } from './LessonComposer';
import type { Composition, LessonDraft, LessonPort } from './types';
import { LessonTabs } from './LessonTabs';
import type { SourcePort } from '../application/ports';
import type { Library } from '../application/contentTypes';
import type { SourcePreview } from '../application/sourceTypes';
const empty = (unitId: string): Composition => ({ contractVersion: 1, unitId, rootNodeId: 'root', nodes: [{ nodeId: 'root', typeId: 'stack', typeVersion: 1, props: { gap: 'normal', headingLevel: 2 }, slots: { body: [] }, on: {} }], bindings: [], sourceBindingRefs: [], appearance: { spacing: 'normal', theme: 'system', typography: 'default', width: 'normal', emphasis: 'normal' }, authoringProvenance: { kind: 'manual' } });
function fixture() {
  const drafts = new Map<string, LessonDraft>();
  const p = { discover: vi.fn(async () => []), read: vi.fn(async () => null), listDrafts: vi.fn(async (unitId: string) => { const draft = { id: unitId, unitId, revision: 1, composition: empty(unitId), baseHeadRevision: null }; drafts.set(unitId, draft); return [draft]; }), readResource: vi.fn(async () => null), readActivity: vi.fn(async () => null), readSource: vi.fn(async () => null), editDraft: vi.fn(async request => { const prior = drafts.get(request.unitId)!; const composition = structuredClone(prior.composition); for (const op of request.operations) { if (op.kind === 'insertNode') { composition.nodes.push(op.node); composition.nodes.find(n => n.nodeId === op.parent)!.slots[op.slot].splice(op.index, 0, op.node.nodeId); } if (op.kind === 'configureNode') composition.nodes.find(n => n.nodeId === op.id)!.props = op.props; if (op.kind === 'setAppearance') composition.appearance = op.tokens; } const next = { ...prior, revision: prior.revision + 1, composition }; drafts.set(request.unitId, next); return next; }) } as unknown as LessonPort; return p;
}
it('provides real manual node forms and isolates English and physics drafts with reused root IDs', async () => {
  const port = fixture(); render(<><section aria-label="English"><LessonComposer workspaceId="workspace" unitId="english" title="English lesson" port={port} /></section><section aria-label="Physics"><LessonComposer workspaceId="workspace" unitId="physics" title="Physics lesson" port={port} /></section></>);
  const english = within(screen.getByRole('region', { name: 'English' })); const physics = within(screen.getByRole('region', { name: 'Physics' }));
  await english.findByRole('button', { name: 'Add component' });
  fireEvent.change(english.getByLabelText('Component type'), { target: { value: 'section' } }); fireEvent.click(english.getByRole('button', { name: 'Add component' }));
  await english.findByLabelText('Section title'); fireEvent.change(english.getByLabelText('Section title'), { target: { value: 'English only' } }); fireEvent.click(english.getByRole('button', { name: 'Save component settings' }));
  await waitFor(() => expect(english.getByRole('button', { name: '2. Section — English only' })).toBeInTheDocument()); expect(physics.queryByText(/English only/)).not.toBeInTheDocument();
  expect(vi.mocked(port.editDraft).mock.calls.every(([request]) => request.unitId === 'english')).toBe(true);
  expect(english.getByRole('button', { name: 'Undo lesson change' })).toBeEnabled();
});
it('moves, nests, removes and undoes through semantic controls while preserving node identities', async () => {
  const p = fixture(); const c = empty('cooking'); c.nodes[0].slots.body = ['a', 'b']; c.nodes.push(...['a', 'b'].map(nodeId => ({ nodeId, typeId: 'section', typeVersion: 1, props: { title: nodeId === 'a' ? 'Prepare' : 'Cook', collapsible: true }, slots: { body: [] }, on: {} })));
  const draft = (composition: Composition, revision: number) => ({ id: 'draft', unitId: 'cooking', revision, composition, baseHeadRevision: null });
  vi.mocked(p.listDrafts).mockResolvedValue([draft(c, 1)]);
  const reordered = structuredClone(c); reordered.nodes[0].slots.body = ['b', 'a'];
  const nested = structuredClone(reordered); nested.nodes[0].slots.body = ['b']; nested.nodes[2].slots.body = ['a'];
  const removed = structuredClone(nested); removed.nodes = removed.nodes.filter(n => n.nodeId !== 'a'); removed.nodes.find(n => n.nodeId === 'b')!.slots.body = [];
  vi.mocked(p.editDraft).mockResolvedValueOnce(draft(reordered, 2)).mockResolvedValueOnce(draft(nested, 3)).mockResolvedValueOnce(draft(removed, 4)).mockResolvedValueOnce(draft(nested, 5));
  render(<LessonComposer workspaceId="w" unitId="cooking" title="Cooking" port={p} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Move 3. Section — Cook up' }));
  await waitFor(() => expect(p.editDraft).toHaveBeenCalledWith(expect.objectContaining({ operations: [{ kind: 'moveNode', id: 'b', parent: 'root', slot: 'body', index: 0 }] })));
  fireEvent.click(screen.getByRole('button', { name: '2. Section — Prepare' })); fireEvent.change(screen.getByLabelText('New parent'), { target: { value: 'b' } }); fireEvent.click(screen.getByRole('button', { name: 'Move into selected parent' }));
  await waitFor(() => expect(vi.mocked(p.editDraft).mock.calls[1][0].operations).toEqual([{ kind: 'moveNode', id: 'a', parent: 'b', slot: 'body', index: 0 }]));
  fireEvent.click(screen.getByRole('button', { name: 'Remove 2. Section — Prepare' })); await waitFor(() => expect(screen.queryByRole('button', { name: '2. Section — Prepare' })).not.toBeInTheDocument()); await waitFor(() => expect(screen.getByRole('button', { name: '2. Section — Cook' })).toHaveFocus());
  fireEvent.click(screen.getByRole('button', { name: 'Undo lesson change' })); await screen.findByRole('button', { name: '2. Section — Prepare' }); expect(vi.mocked(p.editDraft).mock.calls[3][0].operations).toContainEqual(expect.objectContaining({ kind: 'insertSubtree', rootNodeId: 'a' }));
});
it('retains unfinished forms on tab switches and close/reopen, with keyboard tab navigation', async () => {
  const p = fixture(); const data = { workspaceId: 'w', units: { english: { id: 'english', title: 'English', archived: false }, physics: { id: 'physics', title: 'Physics', archived: false } }, resources: {} } as unknown as Library;
  render(<LessonTabs data={data} port={p} sources={{} as SourcePort} perform={vi.fn()} />);
  fireEvent.change(screen.getByLabelText('Existing learning unit'), { target: { value: 'english' } }); fireEvent.click(screen.getByRole('button', { name: 'Open lesson' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Configure selected component' })); fireEvent.change(screen.getByLabelText('Stack gap'), { target: { value: 'wide' } });
  fireEvent.change(screen.getByLabelText('Existing learning unit'), { target: { value: 'physics' } }); fireEvent.click(screen.getByRole('button', { name: 'Open lesson' })); await screen.findByRole('region', { name: 'Physics' });
  fireEvent.keyDown(screen.getByRole('tab', { name: 'Physics' }), { key: 'ArrowLeft' }); expect(screen.getByRole('tab', { name: 'English' })).toHaveFocus(); expect(screen.getByLabelText('Stack gap')).toHaveValue('wide');
  fireEvent.click(screen.getByRole('button', { name: 'Close current lesson' })); fireEvent.change(screen.getByLabelText('Existing learning unit'), { target: { value: 'english' } }); fireEvent.click(screen.getByRole('button', { name: 'Open lesson' }));
  expect(screen.getByLabelText('Stack gap')).toHaveValue('wide'); expect(p.listDrafts).toHaveBeenCalledTimes(2);
});
it('preserves a newer local settings form when its saved operation loses the reply', async () => {
  const p = fixture(); vi.mocked(p.editDraft).mockRejectedValueOnce(new Error('reply lost'));
  render(<LessonComposer workspaceId="w" unitId="python" title="Python" port={p} />); fireEvent.click(await screen.findByRole('button', { name: 'Configure selected component' }));
  fireEvent.change(screen.getByLabelText('Stack gap'), { target: { value: 'wide' } }); fireEvent.click(screen.getByRole('button', { name: 'Save component settings' }));
  await screen.findByRole('button', { name: 'Retry exact pending change' }); expect(screen.getByLabelText('Stack gap')).toHaveValue('wide'); expect(screen.getByRole('button', { name: 'Save component settings' })).toBeDisabled();
  const original = vi.mocked(p.editDraft).mock.calls[0][0]; fireEvent.click(screen.getByRole('button', { name: 'Retry exact pending change' })); await waitFor(() => expect(p.editDraft).toHaveBeenCalledTimes(2)); expect(vi.mocked(p.editDraft).mock.calls[1][0]).toEqual(original);
});
it('opens an earlier saved snapshot through its exact Unit-scoped revision, retaining inert unsupported payloads', async () => {
  const p = fixture(); const saved = { id: 'current', unitId: 'english', composition: empty('english'), parentRevisionId: 'original', registryContractVersion: 1 as const, hash: 'hash' }; const original = { ...saved, id: 'original', parentRevisionId: null, composition: empty('english') }; original.composition.nodes[0].typeId = 'future';
  vi.mocked(p.read).mockResolvedValue(saved); vi.mocked(p.listDrafts).mockResolvedValue([]); p.readRevision = vi.fn(async () => original);
  render(<LessonComposer workspaceId="w" unitId="english" title="English" port={p} />); fireEvent.click(await screen.findByRole('button', { name: 'Saved lesson' }));
  fireEvent.click(await screen.findByRole('button', { name: 'View earlier saved snapshot' })); await screen.findByText(/Unsupported component preserved/);
  expect(p.readRevision).toHaveBeenCalledWith('english', 'original'); expect(screen.getByRole('heading', { name: 'Earlier saved snapshot — read only' })).toBeInTheDocument(); expect(p.editDraft).not.toHaveBeenCalled();
});

function savedSourceFixture() {
  const p = fixture(); const composition = empty('english');
  composition.nodes[0].slots.body = ['source'];
  composition.nodes.push({ nodeId: 'source', typeId: 'source', typeVersion: 1, props: { bindingId: 'original' }, slots: {}, on: {} });
  composition.bindings.push({ bindingId: 'original', targetKind: 'source', targetId: 'source-id', revisionId: 'snapshot' });
  vi.mocked(p.read).mockResolvedValue({ id: 'saved', unitId: 'english', composition, parentRevisionId: null, registryContractVersion: 1, hash: 'hash' });
  vi.mocked(p.listDrafts).mockResolvedValue([]);
  vi.mocked(p.readSource).mockResolvedValue({ id: 'snapshot', unitId: 'english', sourceId: 'source-id', assetDigest: 'digest', filename: 'original.html', attribution: null, license: null, originUrl: null, declaredUsedRanges: [], extractionCoverage: 'notExtracted', readCoverage: 'unknown', warnings: [] });
  const preview: SourcePreview = { source: { id: 'source-id', resourceId: 'resource', digest: 'digest', filename: 'original.html', mediaType: 'text/html', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 13 }, text: '<p>Source</p>', hex: null, truncated: false };
  const sourcePreview = vi.fn(async () => preview);
  return { p, preview, sourcePreview, sources: { sourcePreview } as unknown as SourcePort };
}
it('returns keyboard focus to the lesson original opener after closing its passive preview', async () => {
  const { p, sources } = savedSourceFixture();
  render(<LessonComposer workspaceId="w" unitId="english" title="English" port={p} sourcePort={sources} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Saved lesson' }));
  const opener = await screen.findByRole('button', { name: 'Open original original.html' }); opener.focus(); fireEvent.click(opener);
  const close = await screen.findByRole('button', { name: 'Close original preview' }); close.focus(); fireEvent.click(close);
  expect(opener).toHaveFocus(); expect(screen.queryByText('<p>Source</p>')).not.toBeInTheDocument();
});
it('does not steal focus when a pending original resolves after its lesson becomes hidden', async () => {
  const { p, sources, sourcePreview, preview } = savedSourceFixture(); let resolve!: (value: SourcePreview) => void;
  sourcePreview.mockImplementationOnce(() => new Promise<SourcePreview>(done => { resolve = done; }));
  const view = (hidden: boolean) => <><div hidden={hidden}><LessonComposer workspaceId="w" unitId="english" title="English" port={p} sourcePort={sources} /></div><button>Other lesson control</button></>;
  const { rerender } = render(view(false)); fireEvent.click(await screen.findByRole('button', { name: 'Saved lesson' }));
  const opener = await screen.findByRole('button', { name: 'Open original original.html' }); opener.focus(); fireEvent.click(opener);
  rerender(view(true)); const other = screen.getByRole('button', { name: 'Other lesson control' }); other.focus(); resolve(preview);
  await waitFor(() => expect(screen.getByText('<p>Source</p>')).toBeInTheDocument()); expect(other).toHaveFocus();
});
