import { beforeEach, expect, it, vi } from 'vitest';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LibraryApp } from './LibraryApp';
import type { Library } from './contentBridge';

const calls = vi.hoisted(() => ({ library: vi.fn(), applyContent: vi.fn(), searchResources: vi.fn(), invoke: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: calls.invoke }));
vi.mock('./contentBridge', async importOriginal => ({ ...await importOriginal<typeof import('./contentBridge')>(), ...calls }));
vi.mock('./useReadingPreferences', () => ({ useReadingPreferences: () => ({ preferences: { theme: 'system', textScale: 100 }, status: 'Saved', loading: false, saving: false, dirty: false, error: null, update: vi.fn(), save: vi.fn(), reset: vi.fn() }) }));
vi.mock('./ReadingPreferencesPanel', () => ({ ReadingPreferencesPanel: () => <div /> }));
vi.mock('./MaterialPanel', () => ({ MaterialPanel: ({ resourceId }: { resourceId: string }) => <section aria-label="Open material">{resourceId}</section> }));
beforeEach(() => { vi.clearAllMocks(); });
it('creates an arbitrary blank subject without a template using the common typed contract', async () => {
  const snapshot: Library = { workspaceId: 'workspace', revision: 0, subjects: {}, goals: {}, collections: {}, resources: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  calls.library.mockResolvedValue(snapshot);
  calls.applyContent.mockImplementation(async request => ({ ...snapshot, revision: 1, subjects: { [request.command.id]: { id: request.command.id, title: request.command.title, priorKnowledge: null, archived: false } } }));
  render(<LibraryApp />); const user = userEvent.setup();
  await user.type(await screen.findByLabelText('New subject'), 'Quantum physics');
  await user.click(screen.getByRole('button', { name: 'Create subject' }));
  await waitFor(() => expect(calls.applyContent).toHaveBeenCalledWith(expect.objectContaining({ expectedRevision: 0, command: expect.objectContaining({ type: 'createSubject', title: 'Quantum physics' }) })));
  expect(await screen.findByRole('button', { name: 'Quantum physics' })).toBeVisible();
});

function blank(): Library { return { workspaceId: 'workspace', revision: 0, subjects: {}, goals: {}, collections: {}, resources: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} }; }
it('keeps an unconfirmed organization request through reload and blocks replacement commands until exact retry', async () => {
  calls.library.mockResolvedValue(blank());
  calls.applyContent.mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockResolvedValue({ ...blank(), revision: 1 });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.type(await screen.findByLabelText('New subject'), 'Quantum physics');
  await user.click(screen.getByRole('button', { name: 'Create subject' }));
  await screen.findByRole('button', { name: 'Retry last change' });
  await user.click(screen.getByRole('button', { name: 'Reload organization' }));
  expect(await screen.findByRole('button', { name: 'Retry last change' })).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Create subject' }));
  expect(calls.applyContent).toHaveBeenCalledTimes(1);
  await user.click(screen.getByRole('button', { name: 'Retry last change' }));
  expect(calls.applyContent.mock.calls[1][0]).toEqual(calls.applyContent.mock.calls[0][0]);
});
it('searches explicitly, follows opaque cursors even after an empty page, and keeps the current material', async () => {
  calls.library.mockResolvedValue({ ...blank(), resources: { r: { id: 'r', title: 'Current', archived: false } } });
  calls.searchResources.mockResolvedValueOnce({ revision: 0, items: [], nextOffset: 200 }).mockResolvedValueOnce({ revision: 0, items: [{ resourceId: 'r', title: 'Current', snippet: 'A canonical text match' }], nextOffset: null });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Current' }));
  await user.type(screen.getByLabelText('Search titles, tags and text'), 'canonical');
  expect(calls.searchResources).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Search materials' }));
  expect(await screen.findByText('No matches in this batch. More materials remain to search.')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Continue search' }));
  expect(await screen.findByText('A canonical text match')).toBeVisible();
  expect(calls.searchResources.mock.calls).toEqual([['canonical', 0], ['canonical', 200]]);
  expect(screen.getByRole('region', { name: 'Open material' })).toHaveTextContent('r');
  expect(screen.getByRole('heading', { name: 'Search results' })).toHaveFocus();
});
it('shows accessible pagination and restores archived resources with their stable IDs', async () => {
  const resources = Object.fromEntries(Array.from({ length: 21 }, (_, i) => [`r${i}`, { id: `r${i}`, title: `Material ${i}`, archived: false }]));
  const snapshot = { ...blank(), resources: { ...resources, archived: { id: 'archived', title: 'Saved material', archived: true } } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1 });
  render(<LibraryApp />); const user = userEvent.setup();
  await screen.findByRole('button', { name: 'Material 0' });
  await user.click(screen.getByRole('button', { name: 'Next materials' }));
  expect(screen.getByRole('button', { name: 'Material 20' })).toBeVisible();
  expect(screen.getByRole('heading', { name: 'Available materials' })).toHaveFocus();
  await user.click(screen.getByText('Manage archived items'));
  await user.click(screen.getByRole('button', { name: 'Restore material Saved material' }));
  expect(calls.applyContent).toHaveBeenCalledWith(expect.objectContaining({ command: { type: 'archive', target: { kind: 'resource', id: 'archived' }, archived: false } }));
});

it('ignores an outdated search response after the query changes and retains errors for retry', async () => {
  calls.library.mockResolvedValue(blank());
  let finish!: (page: { revision: number; items: { resourceId: string; title: string; snippet: string }[]; nextOffset: null }) => void;
  calls.searchResources.mockReturnValueOnce(new Promise(resolve => { finish = resolve; })).mockRejectedValueOnce({ code: 'permission_denied' }).mockResolvedValueOnce({ revision: 0, items: [], nextOffset: null });
  render(<LibraryApp />); const user = userEvent.setup();
  const input = await screen.findByLabelText('Search titles, tags and text');
  await user.type(input, 'old'); await user.click(screen.getByRole('button', { name: 'Search materials' }));
  await user.clear(input); await user.type(input, 'new');
  await act(async () => { finish({ revision: 0, items: [{ resourceId: 'hidden', title: 'Old result', snippet: 'old' }], nextOffset: null }); });
  expect(screen.queryByRole('button', { name: 'Old result' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Search materials' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('outside this workspace');
  expect(input).toHaveValue('new');
  await user.click(screen.getByRole('button', { name: 'Search materials' }));
  expect(await screen.findByText('No matches in this batch. Search complete.')).toBeVisible();
});

it('reorders a collection by keyboard using stable IDs and retains the focused control', async () => {
  const snapshot: Library = { ...blank(), subjects: { s: { id: 's', title: 'Any subject', priorKnowledge: null, archived: false } }, collections: { first: { id: 'first', title: 'First', subjectId: 's', parentId: null, position: 0, archived: false }, second: { id: 'second', title: 'Second', subjectId: 's', parentId: null, position: 1, archived: false } } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1, collections: { ...snapshot.collections, first: { ...snapshot.collections.first, position: 1 }, second: { ...snapshot.collections.second, position: 0 } } });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Any subject' }));
  const move = screen.getByRole('button', { name: 'Move First later' }); move.focus(); await user.keyboard('{Enter}');
  await waitFor(() => expect(calls.applyContent).toHaveBeenCalledWith(expect.objectContaining({ command: { type: 'reorderCollections', subjectId: 's', parentId: null, orderedIds: ['second', 'first'] } })));
  expect(screen.getByRole('button', { name: 'Move First earlier' })).toHaveFocus();
});
it('edits a learning unit objective and its explicit goal and shared resource links', async () => {
  const snapshot = { ...blank(), goals: { g: { id: 'g', title: 'Explain', subjectId: 's', criteria: 'In own words', archived: false } }, resources: { r: { id: 'r', title: 'Source', archived: false } }, units: { u: { id: 'u', title: 'Practice', objective: 'Old objective', goalIds: [], resourceIds: [], archived: false } } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1 });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.selectOptions(await screen.findByLabelText('Edit learning unit'), 'u');
  await user.clear(screen.getByLabelText('Unit objective')); await user.type(screen.getByLabelText('Unit objective'), 'Explain a new example');
  await user.click(screen.getByRole('checkbox', { name: 'Goal: Explain' })); await user.click(screen.getByRole('checkbox', { name: 'Material: Source' }));
  await user.click(screen.getByRole('button', { name: 'Save unit' }));
  expect(calls.applyContent).toHaveBeenCalledWith(expect.objectContaining({ command: { type: 'updateUnit', id: 'u', title: 'Practice', objective: 'Explain a new example', goalIds: ['g'], resourceIds: ['r'] } }));
});

it('passes literal search syntax and the opaque offset as native parameters', async () => {
  const { createNativeContentPort } = await import('./application/nativeContentPort');
  const bridge = createNativeContentPort('test-course');
  const result = { revision: 7, items: [], nextOffset: 400 }; calls.invoke.mockResolvedValue(result);
  expect(await bridge.searchResources("Ä%_' OR (.*)", 200)).toBe(result);
  expect(calls.invoke).toHaveBeenCalledWith('search_resources', { courseId: 'test-course', query: "Ä%_' OR (.*)", offset: 200 });
});
it('limits search by Unicode scalars and discards results when the library revision changes', async () => {
  const snapshot: Library = { ...blank(), resources: { r: { id: 'r', title: 'Current', archived: false } } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1 });
  calls.searchResources.mockResolvedValue({ revision: 0, items: [], nextOffset: 200 });
  render(<LibraryApp />); const user = userEvent.setup(); const input = await screen.findByLabelText('Search titles, tags and text');
  fireEvent.change(input, { target: { value: '😀'.repeat(257) } });
  expect(screen.getByRole('button', { name: 'Search materials' })).toBeDisabled();
  fireEvent.change(input, { target: { value: '😀'.repeat(256) } });
  await user.click(screen.getByRole('button', { name: 'Search materials' }));
  expect(calls.searchResources).toHaveBeenCalledWith('😀'.repeat(256), 0);
  expect(await screen.findByRole('button', { name: 'Continue search' })).toBeEnabled();
  await user.click(screen.getByText('Manage archived items')); await user.click(screen.getByRole('button', { name: 'Archive material Current' }));
  await waitFor(() => expect(screen.queryByRole('button', { name: 'Continue search' })).not.toBeInTheDocument());
  expect(input).toHaveValue('😀'.repeat(256));
  calls.searchResources.mockResolvedValueOnce({ revision: 0, items: [{ resourceId: 'old', title: 'Stale result', snippet: 'old' }], nextOffset: null });
  await user.click(screen.getByRole('button', { name: 'Search materials' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('library changed');
  expect(screen.queryByRole('button', { name: 'Stale result' })).not.toBeInTheDocument();
});

it('moves a path step with keyboard controls while preserving step identities', async () => {
  const snapshot: Library = { ...blank(), units: { u: { id: 'u', title: 'Practice', objective: 'Do it', resourceIds: [], goalIds: [], archived: false } }, paths: { p: { id: 'p', title: 'My path', archived: false, steps: [{ id: 'a', unitId: 'u' }, { id: 'b', unitId: 'u' }] } } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1, paths: { p: { ...snapshot.paths.p, steps: [...snapshot.paths.p.steps].reverse() } } });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.selectOptions(await screen.findByLabelText('Path'), 'p');
  screen.getByRole('button', { name: 'Move step 1: Practice later' }).focus(); await user.keyboard('{Enter}');
  expect(calls.applyContent).toHaveBeenCalledWith(expect.objectContaining({ command: { type: 'setPathSteps', id: 'p', steps: [{ id: 'b', unitId: 'u' }, { id: 'a', unitId: 'u' }] } }));
});

it('uses the same subject forms for criteria, collection moves and shared unit/material placements', async () => {
  const snapshot: Library = { ...blank(), subjects: { s: { id: 's', title: 'Ecology', priorKnowledge: null, archived: false }, t: { id: 't', title: 'Music', priorKnowledge: null, archived: false } }, collections: { c: { id: 'c', title: 'Sources', subjectId: 's', parentId: null, position: 0, archived: false }, d: { id: 'd', title: 'Destination', subjectId: 't', parentId: null, position: 0, archived: false } }, resources: { r: { id: 'r', title: 'Shared reading', archived: false } }, units: { u: { id: 'u', title: 'Practice', objective: 'Apply', goalIds: [], resourceIds: [], archived: false } }, placements: { a: { id: 'a', collectionId: 'c', target: { kind: 'resource', id: 'r' }, position: 0, archived: false }, b: { id: 'b', collectionId: 'c', target: { kind: 'unit', id: 'u' }, position: 1, archived: false } } };
  snapshot.goals = { g: { id: 'g', title: 'Explain', subjectId: 's', criteria: 'Use own words', archived: false } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1 });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Ecology' }));
  await user.type(screen.getByLabelText('New goal'), 'Explain'); await user.type(screen.getByLabelText('Goal criteria'), 'Use an unfamiliar example'); await user.click(screen.getByRole('button', { name: 'Add goal' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: expect.objectContaining({ type: 'createGoal', subjectId: 's', title: 'Explain', criteria: 'Use an unfamiliar example' }) }));
  await user.type(screen.getByLabelText('New collection'), 'Examples'); await user.click(screen.getByRole('button', { name: 'Create collection' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: expect.objectContaining({ type: 'createCollection', subjectId: 's', parentId: null }) }));
  await user.click(screen.getByRole('button', { name: 'Sources' }));
  expect(screen.getByRole('heading', { name: 'Collections — Sources' })).toHaveFocus();
  await user.type(screen.getByLabelText('New collection title'), 'Shared sources'); await user.click(screen.getByRole('button', { name: 'Rename collection' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'renameCollection', id: 'c', title: 'Shared sources' } }));
  await user.selectOptions(screen.getByLabelText('Reuse material'), 'r'); await user.click(screen.getByRole('button', { name: 'Add shared placement' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: expect.objectContaining({ type: 'place', collectionId: 'c', target: { kind: 'resource', id: 'r' } }) }));
  await user.selectOptions(screen.getByLabelText('Place learning unit'), 'u'); await user.click(screen.getByRole('button', { name: 'Place unit' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: expect.objectContaining({ type: 'place', collectionId: 'c', target: { kind: 'unit', id: 'u' } }) }));
  await user.click(screen.getByRole('button', { name: 'Move placement of Practice earlier' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'reorderPlacements', collectionId: 'c', orderedIds: ['b', 'a'] } }));
  await user.selectOptions(screen.getByLabelText('Move placement of Shared reading to collection'), 'd'); await user.click(screen.getAllByRole('button', { name: 'Move placement' })[0]);
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'movePlacement', id: 'a', collectionId: 'd' } }));
  await user.click(screen.getAllByRole('button', { name: 'Remove placement' })[0]);
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'archive', target: { kind: 'placement', id: 'a' }, archived: true } }));
  await user.selectOptions(screen.getByLabelText('Move collection to subject'), 't'); await user.selectOptions(screen.getByLabelText('Destination parent'), 'd'); await user.click(screen.getByRole('button', { name: 'Move collection' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'moveCollection', id: 'c', subjectId: 't', parentId: 'd' } }));
  expect(screen.getByRole('heading', { name: 'Collections — subject root' })).toHaveFocus();
  await user.click(screen.getByRole('button', { name: 'Sources' })); await user.click(screen.getByRole('button', { name: 'Archive collection' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'archive', target: { kind: 'collection', id: 'c' }, archived: true } }));
});

it('authors an ordered path through explicit append and remove operations', async () => {
  const snapshot: Library = { ...blank(), units: { u: { id: 'u', title: 'Practice', objective: 'Do it', resourceIds: [], goalIds: [], archived: false } }, paths: { p: { id: 'p', title: 'My path', archived: false, steps: [{ id: 'a', unitId: 'u' }] } } };
  calls.library.mockResolvedValue(snapshot); calls.applyContent.mockResolvedValue({ ...snapshot, revision: 1 });
  render(<LibraryApp />); const user = userEvent.setup();
  await user.type(await screen.findByLabelText('New path'), 'New journey'); await user.click(screen.getByRole('button', { name: 'Create path' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: expect.objectContaining({ type: 'createPath', title: 'New journey', steps: [] }) }));
  await user.selectOptions(screen.getByLabelText('Path'), 'p'); await user.selectOptions(screen.getByLabelText('Next unit'), 'u'); await user.click(screen.getByRole('button', { name: 'Append unit' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: expect.objectContaining({ type: 'setPathSteps', id: 'p', steps: [{ id: 'a', unitId: 'u' }, { id: expect.any(String), unitId: 'u' }] }) }));
  await user.click(screen.getByRole('button', { name: 'Remove step' }));
  expect(calls.applyContent).toHaveBeenLastCalledWith(expect.objectContaining({ command: { type: 'setPathSteps', id: 'p', steps: [] } }));
});

it('explains revision conflicts and invalid edits without exposing native error payloads', async () => {
  const bridge = await vi.importActual<typeof import('./contentBridge')>('./contentBridge');
  expect(bridge.failureMessage({ code: 'revision_conflict', secret: '/private/path' })).toContain('draft is retained');
  expect(bridge.failureMessage({ code: 'invalid_request' })).toContain('Check the fields');
  expect(bridge.failureMessage({ code: 'outcome_unconfirmed' })).toContain('durable receipt');
  expect(bridge.failureMessage(new Error('/private/path'))).not.toContain('/private/path');
});
