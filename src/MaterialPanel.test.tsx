import { expect, it, vi } from 'vitest';
import { StrictMode } from 'react';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MaterialPanel } from './MaterialPanel';
import type { Library, SavedDocument } from './contentBridge';

const bridge = vi.hoisted(() => ({ readDocument: vi.fn(), saveDocument: vi.fn(), readingPosition: vi.fn(), annotations: vi.fn(), saveAnnotation: vi.fn(), saveReadingPosition: vi.fn() }));
vi.mock('./contentBridge', async load => ({ ...await load<typeof import('./contentBridge')>(), ...bridge }));
vi.mock('./sourceBridge', () => ({
  sources: vi.fn().mockResolvedValue({ revision: 0, sources: [] }),
  pickSource: vi.fn().mockResolvedValue({ digest: 'a'.repeat(64), byteLength: 12, filename: 'source.txt' }),
  saveSource: vi.fn(), sourcePreview: vi.fn(),
}));
vi.mock('./DocumentEditor', () => ({ DocumentEditor: ({ document, onChange }: { document: import('./documentModel').Document; onChange: (value: import('./documentModel').Document) => void }) => {
  const block = document.blocks[0]; const text = block.content.type === 'paragraph' ? block.content.inlines.map(x => x.text).join('') : '';
  return <label>Document text<textarea value={text} onChange={e => onChange({ ...document, blocks: [{ ...block, content: { type: 'paragraph', inlines: [{ text: e.target.value, marks: [] }] } }] })} /></label>;
} }));
it('retains failed-save edits, retries the same identity and preserves newer edits after an earlier draft is acknowledged', async () => {
  const initial: SavedDocument = { id: 'revision1', resourceId: 'resource', revision: 1, document: { schemaVersion: 1, blocks: [{ id: 'block', content: { type: 'paragraph', inlines: [{ text: 'original', marks: [] }] } }] } };
  const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  bridge.readDocument.mockResolvedValue(initial); bridge.readingPosition.mockResolvedValue(null); bridge.annotations.mockResolvedValue([]);
  bridge.saveDocument.mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async request => ({ id: 'revision2', resourceId: 'resource', revision: 2, document: request.document }));
  render(<MaterialPanel data={data} resourceId="resource" onDirty={vi.fn()} perform={vi.fn()} />);
  const user = userEvent.setup(); const editor = await screen.findByLabelText('Document text');
  await user.clear(editor); await user.type(editor, 'first draft'); await user.click(screen.getByRole('button', { name: 'Save material' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('unconfirmed'); expect(editor).toHaveValue('first draft');
  expect(screen.getByRole('button', { name: 'Save material' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Discard unsaved changes' })).toBeDisabled();
  expect(screen.getByLabelText('Annotation at start of block')).toBeDisabled();
  await user.clear(editor); await user.type(editor, 'newer draft'); await user.click(screen.getByRole('button', { name: 'Retry same save' }));
  expect(bridge.saveDocument.mock.calls[1][0]).toEqual(bridge.saveDocument.mock.calls[0][0]);
  expect(await screen.findByText(/Newer edits still need saving/)).toBeVisible(); expect(editor).toHaveValue('newer draft');
  expect(screen.getByRole('button', { name: 'Save material' })).toBeEnabled();
});

it('replays annotation and reading-position requests exactly after lost replies without losing newer annotation edits', async () => {
  const initial: SavedDocument = { id: 'revision1', resourceId: 'resource', revision: 1, document: { schemaVersion: 1, blocks: [{ id: 'block', content: { type: 'paragraph', inlines: [{ text: 'original', marks: [] }] } }] } };
  const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  bridge.readDocument.mockResolvedValue(initial); bridge.readingPosition.mockResolvedValue(null); bridge.annotations.mockResolvedValue([]);
  bridge.saveAnnotation.mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async request => ({ ...request.annotation, revision: 1 }));
  bridge.saveReadingPosition.mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async request => ({ ...request.position, revision: 1 }));
  const onDirty = vi.fn(); render(<StrictMode><MaterialPanel data={data} resourceId="resource" onDirty={onDirty} perform={vi.fn()} /></StrictMode>);
  const user = userEvent.setup(); const note = await screen.findByLabelText('Annotation at start of block');
  await user.type(note, 'first note'); await user.click(screen.getByRole('button', { name: 'Add annotation' }));
  expect(await screen.findByRole('button', { name: 'Retry saved context' })).toBeVisible();
  await user.clear(note); await user.type(note, 'newer note');
  expect(onDirty).toHaveBeenLastCalledWith(true);
  expect(screen.getByRole('button', { name: 'Save material' })).toBeDisabled();
  await user.click(screen.getByRole('button', { name: 'Add annotation' })); expect(bridge.saveAnnotation).toHaveBeenCalledTimes(1);
  await user.click(screen.getByRole('button', { name: 'Retry saved context' }));
  expect(bridge.saveAnnotation.mock.calls[1][0]).toEqual(bridge.saveAnnotation.mock.calls[0][0]);
  expect(note).toHaveValue('newer note'); expect(screen.getByText(/first note/)).toBeVisible();
  await user.clear(note); await user.click(screen.getByRole('button', { name: 'Remember reading location' }));
  await screen.findByRole('button', { name: 'Retry saved context' });
  await user.click(screen.getByRole('button', { name: 'Remember reading location' })); expect(bridge.saveReadingPosition).toHaveBeenCalledTimes(1);
  await user.click(screen.getByRole('button', { name: 'Retry saved context' }));
  expect(bridge.saveReadingPosition.mock.calls[1][0]).toEqual(bridge.saveReadingPosition.mock.calls[0][0]);
  expect(await screen.findByText('Reading location saved locally.')).toBeVisible();
  expect(onDirty).toHaveBeenLastCalledWith(false);
});

it('protects attachment drafts and keeps document protection after the attachment finishes', async () => {
  const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  bridge.readDocument.mockResolvedValue(null); bridge.readingPosition.mockResolvedValue(null); bridge.annotations.mockResolvedValue([]);
  const onDirty = vi.fn(); const user = userEvent.setup();
  render(<MaterialPanel data={data} resourceId="resource" onDirty={onDirty} perform={vi.fn()} />);
  const editor = await screen.findByLabelText('Document text');
  await user.click(screen.getByRole('button', { name: 'Choose source file' }));
  expect(onDirty).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole('button', { name: 'Discard selected source' }));
  expect(onDirty).toHaveBeenLastCalledWith(false);
  await user.type(editor, 'unsaved document');
  await user.click(screen.getByRole('button', { name: 'Choose source file' }));
  await user.click(screen.getByRole('button', { name: 'Discard selected source' }));
  expect(onDirty).toHaveBeenLastCalledWith(true);
});

it('locks revision-bound context while a document save is awaiting its reply', async () => {
  const initial: SavedDocument = { id: 'revision1', resourceId: 'resource', revision: 1, document: { schemaVersion: 1, blocks: [{ id: 'block', content: { type: 'paragraph', inlines: [{ text: 'original', marks: [] }] } }] } };
  const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  bridge.readDocument.mockResolvedValue(initial); bridge.readingPosition.mockResolvedValue(null); bridge.annotations.mockResolvedValue([]);
  let resolve!: (value: SavedDocument) => void;
  bridge.saveDocument.mockReturnValueOnce(new Promise<SavedDocument>(done => { resolve = done; }));
  render(<MaterialPanel data={data} resourceId="resource" onDirty={vi.fn()} perform={vi.fn()} />);
  const user = userEvent.setup(); await user.type(await screen.findByLabelText('Document text'), ' draft');
  await user.click(screen.getByRole('button', { name: 'Save material' }));
  expect(screen.getByLabelText('Annotation at start of block')).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Remember reading location' })).toBeDisabled();
  await act(async () => resolve({ ...initial, id: 'revision2', revision: 2 }));
  expect(await screen.findByLabelText('Annotation at start of block')).toBeEnabled();
});

it('keeps a rejected document draft while explicitly refreshing its saved revision before another save', async () => {
  const document = { schemaVersion: 1 as const, blocks: [{ id: 'block', content: { type: 'paragraph' as const, inlines: [{ text: 'original', marks: [] }] } }] };
  const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  bridge.readDocument.mockResolvedValueOnce({ id: 'revision1', resourceId: 'resource', revision: 1, document }).mockResolvedValueOnce({ id: 'revision2', resourceId: 'resource', revision: 2, document });
  bridge.readingPosition.mockResolvedValue(null); bridge.annotations.mockResolvedValue([]);
  bridge.saveDocument.mockRejectedValueOnce({ code: 'revision_conflict' }).mockImplementation(async request => ({ id: 'revision3', resourceId: 'resource', revision: 3, document: request.document }));
  render(<MaterialPanel data={data} resourceId="resource" onDirty={vi.fn()} perform={vi.fn()} />);
  const user = userEvent.setup(); const editor = await screen.findByLabelText('Document text');
  await user.clear(editor); await user.type(editor, 'retained draft'); await user.click(screen.getByRole('button', { name: 'Save material' }));
  await user.click(await screen.findByRole('button', { name: 'Reload saved revision; keep draft' }));
  expect(await screen.findByText(/Latest saved revision loaded/)).toBeVisible(); expect(editor).toHaveValue('retained draft');
  await user.click(screen.getByRole('button', { name: 'Save material' }));
  expect(bridge.saveDocument.mock.calls.at(-1)?.[0]).toEqual(expect.objectContaining({ expectedRevision: 2 }));
});
