import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { NodeEditor } from './NodeEditor';
import { composition } from './testFixtures';
import type { Binding, LessonNode, LessonOperation } from './types';
import type { ResolvedLesson } from './LessonRenderer';

function fixture(kind: 'resource' | 'activity') {
  const c = composition(); const bindings: Binding[] = ['a', 'b'].map(id => ({ bindingId: id, targetKind: kind, targetId: `${kind}-${id}`, revisionId: `revision-${id}` })); c.bindings = bindings;
  const node: LessonNode = { nodeId: 'editor', typeId: kind === 'resource' ? 'content' : 'activity', typeVersion: 1, props: { bindingId: 'a', ...(kind === 'activity' ? { presentation: 'default' } : {}) }, slots: {}, on: {} }; c.nodes.push(node); c.nodes[0].slots.body = [node.nodeId];
  const resolved: ResolvedLesson = { resources: {}, activities: {}, sources: {} };
  for (const id of ['a', 'b']) {
    if (kind === 'resource') resolved.resources[`revision-${id}`] = { id: `revision-${id}`, resourceId: `resource-${id}`, revision: 1, document: { schemaVersion: 1, blocks: [{ id: `block-${id}`, content: { type: 'paragraph', inlines: [{ text: `Original ${id}`, marks: [] }] } }] } };
    else resolved.activities[`revision-${id}`] = { id: `revision-${id}`, activityId: `activity-${id}`, unitId: c.unitId, definition: { family: 'recall', version: 1, prompt: `Prompt ${id}`, answer: `Answer ${id}`, hints: [], evaluation: 'self' } };
  }
  return { c, node, resolved };
}
async function editPayload(kind: 'resource' | 'activity') {
  if (kind === 'resource') { const editor = await screen.findByRole('textbox', { name: 'Lesson material 2' }); editor.focus(); fireEvent.paste(editor, { clipboardData: { getData: () => 'Edited A only ' } }); }
  else fireEvent.change(await screen.findByLabelText('Exercise prompt'), { target: { value: 'Edited A only' } });
}
const saveLabel = (kind: 'resource' | 'activity') => kind === 'resource' ? 'Save material revision for this lesson' : 'Save exercise revision for this lesson';
function assertPayload(kind: 'resource' | 'activity') { if (kind === 'resource') expect(screen.getByRole('textbox', { name: 'Lesson material 2' })).toHaveTextContent('Edited A only'); else expect(screen.getByLabelText('Exercise prompt')).toHaveValue('Edited A only'); }

it.each(['resource', 'activity'] as const)('anchors %s payload to the persisted pin while a different component pin is only an unsaved choice', async kind => {
  const { c, node, resolved } = fixture(kind); const save = vi.fn(async (_operations: LessonOperation[]) => false);
  render(<NodeEditor node={node} composition={c} resolved={resolved} disabled={false} save={save} />);
  await screen.findByLabelText(kind === 'resource' ? 'Lesson material title' : 'Exercise prompt');
  const pin = screen.getByLabelText(kind === 'resource' ? 'Pinned material' : 'Pinned exercise'); fireEvent.change(pin, { target: { value: 'b' } });
  await editPayload(kind); fireEvent.click(screen.getByRole('button', { name: saveLabel(kind) }));
  await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  expect(save.mock.calls[0][0]).toEqual([expect.objectContaining({ kind: kind === 'resource' ? 'createResourceRevision' : 'createActivityRevision', [kind === 'resource' ? 'resourceId' : 'activityId']: `${kind}-a`, expectedRevisionId: 'revision-a', bindingId: 'a' })]);
  const operation = save.mock.calls[0][0][0];
  if (operation.kind === 'createResourceRevision') expect(operation.document.blocks[0]).toMatchObject({ id: 'block-a', content: { inlines: expect.arrayContaining([expect.objectContaining({ text: expect.stringContaining('Edited A only') })]) } });
  if (operation.kind === 'createActivityRevision') expect(operation.definition).toMatchObject({ prompt: 'Edited A only', answer: 'Answer a' });
  assertPayload(kind); expect(screen.getByRole('button', { name: 'Save component settings' })).toBeDisabled();
});
it.each(['resource', 'activity'] as const)('blocks dirty %s rebind and preserves its target, base and payload after rejected and uncertain saves', async kind => {
  const { c, node, resolved } = fixture(kind); const save = vi.fn().mockResolvedValueOnce(false).mockRejectedValueOnce(new Error('Unconfirmed reply'));
  render(<NodeEditor node={node} composition={c} resolved={resolved} disabled={false} save={save} />); await editPayload(kind);
  const pin = screen.getByLabelText(kind === 'resource' ? 'Pinned material' : 'Pinned exercise'); expect(pin).toBeDisabled(); fireEvent.change(pin, { target: { value: 'b' } }); expect(pin).toHaveValue('a');
  const submit = screen.getByRole('button', { name: saveLabel(kind) }); fireEvent.click(submit); await waitFor(() => expect(save).toHaveBeenCalledTimes(1)); assertPayload(kind); expect(pin).toBeDisabled();
  fireEvent.click(submit); await waitFor(() => expect(save).toHaveBeenCalledTimes(2)); await screen.findByText(/could not be confirmed/i); assertPayload(kind); expect(pin).toHaveValue('a'); expect(pin).toBeDisabled();
  for (const [operations] of save.mock.calls) expect(operations[0]).toMatchObject({ [kind === 'resource' ? 'resourceId' : 'activityId']: `${kind}-a`, expectedRevisionId: 'revision-a', bindingId: 'a' });
  fireEvent.click(screen.getByRole('button', { name: kind === 'resource' ? 'Discard material edits and reload saved pin' : 'Discard exercise edits and reload saved pin' }));
  await waitFor(() => expect(pin).toBeEnabled());
  if (kind === 'resource') expect(screen.getByRole('textbox', { name: 'Lesson material 2' })).toHaveTextContent('Original a'); else expect(screen.getByLabelText('Exercise prompt')).toHaveValue('Prompt a');
});
