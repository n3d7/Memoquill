import { expect, it, vi } from 'vitest';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MaterialPanel } from '../MaterialPanel';
import type { Library, SaveDocument, SavedDocument } from '../contentBridge';
import type { useResourceDocument } from '../useResourceDocument';
import type { Document } from '../documentModel';
import { ApplicationProvider } from './ApplicationProvider';
import { createTestContentPort } from './testContentPort';

const native = vi.hoisted(() => vi.fn().mockRejectedValue(new Error('Native transport is unavailable')));
vi.mock('@tauri-apps/api/core', () => ({ invoke: native }));
const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
const document = (text: string): Document => ({ schemaVersion: 1, blocks: [{ id: 'block', content: { type: 'paragraph', inlines: [{ text, marks: [] }] } }] });
function Replacement({ material }: { material: ReturnType<typeof useResourceDocument> }) {
  return <section aria-label="Replacement material"><label>Replacement text<input value={material.document.blocks[0].content.type === 'paragraph' ? material.document.blocks[0].content.inlines[0]?.text ?? '' : ''} onChange={event => material.change(document(event.target.value))} /></label>
    <p role="status">{material.message}</p><button disabled={material.loading || material.busy || material.hasPending} onClick={() => void material.save()}>Save replacement</button>
    <button disabled={material.busy || material.hasPending || !material.dirty} onClick={material.discard}>Discard replacement</button>
    <button onClick={() => void material.retry()}>Retry replacement</button></section>;
}
it('replaces the material view and transport while preserving exact retry and newer draft protection', async () => {
  const requests: SaveDocument[] = [];
  let resolve!: (value: SavedDocument) => void;
  const adapter = createTestContentPort({ library: data, documents: [{ id: 'saved-1', resourceId: 'resource', revision: 1, document: document('original') }] });
  adapter.loseNextSaveReply();
  const port = { ...adapter.port, saveDocument: async (request: SaveDocument) => {
    requests.push(request); await adapter.port.saveDocument(request);
    return new Promise<SavedDocument>(done => { resolve = done; });
  } };
  const props = { data, resourceId: 'resource', onDirty: vi.fn(), perform: vi.fn(), view: Replacement };
  render(<ApplicationProvider ports={{ content: port }}><MaterialPanel {...props} /></ApplicationProvider>);
  const user = userEvent.setup();
  const editor = await screen.findByLabelText('Replacement text');
  await screen.findByText('Saved material opened.');
  await user.clear(editor); await user.type(editor, 'first');
  await user.click(screen.getByRole('button', { name: 'Save replacement' }));
  await screen.findByText(/unconfirmed/);
  expect(screen.getByRole('button', { name: 'Discard replacement' })).toBeDisabled();
  expect(props.onDirty).toHaveBeenLastCalledWith(true);
  await user.clear(editor); await user.type(editor, 'newer');
  await user.click(screen.getByRole('button', { name: 'Retry replacement' }));
  expect(requests[1]).toBe(requests[0]);
  await act(async () => resolve({ id: 'saved-2', resourceId: 'resource', revision: 2, document: requests[0].document }));
  expect(editor).toHaveValue('newer');
  expect(screen.getByRole('status')).toHaveTextContent('Newer edits still need saving.');
  expect(screen.getByRole('button', { name: 'Save replacement' })).toBeEnabled();
  expect(native).not.toHaveBeenCalled();
});
