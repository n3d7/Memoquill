import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { LessonComposer } from './LessonComposer';
import type { Composition, LessonDraft, LessonPort } from './types';
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
  await waitFor(() => expect(english.getByRole('button', { name: '2. section — English only' })).toBeInTheDocument()); expect(physics.queryByText(/English only/)).not.toBeInTheDocument();
  expect(vi.mocked(port.editDraft).mock.calls.every(([request]) => request.unitId === 'english')).toBe(true);
  expect(english.getByRole('button', { name: 'Undo lesson change' })).toBeEnabled();
});
