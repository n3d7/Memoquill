import { expect, it, vi } from 'vitest';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MaterialPanel, type MaterialViewProps } from '../MaterialPanel';
import { ApplicationProvider } from './ApplicationProvider';
import { createTestContentPort } from './testContentPort';
import type { Library, SavedDocument } from './contentTypes';
import type { SelectedSource, SaveSource } from './sourceTypes';
import type { ContentPort } from './ports';

vi.mock('../DocumentEditor', () => ({ DocumentEditor: () => <p>Document editor</p> }));
const data: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
const saved: SavedDocument = { id: 'revision1', resourceId: 'resource', revision: 1, document: { schemaVersion: 1, blocks: [{ id: 'block', content: { type: 'paragraph', inlines: [{ text: 'original', marks: [] }] } }] } };
const selected: SelectedSource = { digest: 'a'.repeat(64), byteLength: 12, filename: 'source.txt' };
function Alternative({ context, sources }: MaterialViewProps) {
  return <section aria-label="Alternative material view">
    <label>Alternative annotation<textarea value={context.body} onChange={event => context.setBody(event.target.value)} /></label>
    <p>Alternative source attribution: {sources.state.draft?.attribution}</p>
    <button onClick={() => void context.retry()}>Retry alternate context</button>
    <button onClick={() => void sources.attach()}>Retry alternate source</button>
  </section>;
}
function setup() {
  const content: ContentPort = { ...createTestContentPort({ library: data, documents: [saved] }).port, readingPosition: vi.fn().mockResolvedValue(null), annotations: vi.fn().mockResolvedValue([]), saveAnnotation: vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async request => ({ ...request.annotation, revision: 1 })) };
  const source = { sources: vi.fn().mockResolvedValue({ revision: 0, sources: [] }), pickSource: vi.fn().mockResolvedValue(selected), saveSource: vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async (request: SaveSource) => ({ ...request.source, revision: 1, byteLength: 12 })), sourcePreview: vi.fn() };
  const onDirty = vi.fn();
  const scene = (view?: typeof Alternative) => <ApplicationProvider ports={{ content, sources: source }}><MaterialPanel data={data} resourceId="resource" perform={vi.fn()} onDirty={onDirty} view={view} /></ApplicationProvider>;
  const rendered = render(scene());
  return { content, source, onDirty, replace: () => rendered.rerender(scene(Alternative)), restore: () => rendered.rerender(scene()) };
}
it('retains an unresolved annotation request and newer draft across replacement, keeping navigation protected', async () => {
  const scenario = setup(); const user = userEvent.setup();
  const note = await screen.findByLabelText('Annotation at start of block');
  await user.type(note, 'first note'); await user.click(screen.getByRole('button', { name: 'Add annotation' }));
  await screen.findByRole('button', { name: 'Retry saved context' });
  await user.clear(note); await user.type(note, 'newer note');
  scenario.replace();
  expect(scenario.onDirty).toHaveBeenLastCalledWith(true);
  expect(screen.getByLabelText('Alternative annotation')).toHaveValue('newer note');
  await user.type(screen.getByLabelText('Alternative annotation'), ' retained');
  scenario.restore();
  expect(await screen.findByLabelText('Annotation at start of block')).toHaveValue('newer note retained');
  await user.click(screen.getByRole('button', { name: 'Retry saved context' }));
  const save = vi.mocked(scenario.content.saveAnnotation);
  expect(save.mock.calls[1][0]).toBe(save.mock.calls[0][0]);
  expect(screen.getByLabelText('Annotation at start of block')).toHaveValue('newer note retained');
  expect(scenario.onDirty).toHaveBeenLastCalledWith(true);
  expect(screen.getByRole('button', { name: 'Save material' })).toBeDisabled();
});
it('retains selected source metadata and exact unconfirmed attachment across replacement', async () => {
  const scenario = setup(); const user = userEvent.setup();
  await screen.findByLabelText('Annotation at start of block');
  await user.click(screen.getByRole('button', { name: 'Choose source file' }));
  await user.type(await screen.findByLabelText('Attribution'), 'Original author');
  await user.click(screen.getByRole('button', { name: 'Attach selected source' }));
  await screen.findByRole('button', { name: 'Retry same attachment' });
  scenario.replace();
  expect(scenario.onDirty).toHaveBeenLastCalledWith(true);
  const close = new Event('beforeunload', { cancelable: true }); window.dispatchEvent(close);
  expect(close.defaultPrevented).toBe(true);
  expect(screen.getByText('Alternative source attribution: Original author')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Retry alternate source' }));
  scenario.restore();
  expect(scenario.source.saveSource.mock.calls[1][0]).toBe(scenario.source.saveSource.mock.calls[0][0]);
  expect(await screen.findByText('Attached original source.')).toBeVisible();
  expect(screen.getByText('Original author')).toBeVisible();
  expect(scenario.onDirty).toHaveBeenLastCalledWith(false);
});
it('retains a picker reply while the trigger is absent and restores focus when the view returns', async () => {
  const scenario = setup(); const user = userEvent.setup();
  let resolve!: (value: SelectedSource) => void;
  scenario.source.pickSource.mockReturnValueOnce(new Promise<SelectedSource>(done => { resolve = done; }));
  await screen.findByLabelText('Annotation at start of block');
  await user.click(screen.getByRole('button', { name: 'Choose source file' }));
  scenario.replace();
  await act(async () => resolve(selected));
  expect(scenario.onDirty).toHaveBeenLastCalledWith(true);
  scenario.restore();
  expect(await screen.findByLabelText('Attribution')).toHaveValue('');
  expect(screen.getByRole('button', { name: 'Choose source file' })).toHaveFocus();
});
