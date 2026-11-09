import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import { CourseApp } from './CourseApp';
import { createNativeCoursePorts } from './nativeCoursePort';
import type { CourseCatalogue, CoursePort } from './types';
import type { Library } from '../application/contentTypes';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const catalogue = (id = 'a'): CourseCatalogue => ({ contractVersion: 1, selectedCourseId: id, courses: [{ id: 'a', title: '<script>A</script>' }, { id: 'b', title: 'Course B' }] });
const blank = (id: string): Library => ({ workspaceId: id, revision: 0, subjects: {}, goals: {}, collections: {}, resources: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} });
function fixtures() {
  const port: CoursePort = { list: vi.fn().mockResolvedValue(catalogue()), select: vi.fn().mockImplementation(async id => catalogue(id)), create: vi.fn() };
  const ports = new Map(['a', 'b'].map(id => {
    const native = createNativeCoursePorts(id);
    return [id, { ...native, lessons: undefined, ingestion: { ...native.ingestion!, list: vi.fn().mockResolvedValue([]) }, content: { ...native.content, library: vi.fn().mockResolvedValue(blank(id)), applyContent: vi.fn(), searchResources: vi.fn() }, workspace: { ...native.workspace, workspaceStatus: vi.fn().mockResolvedValue({ contractVersion: 1, preferences: { workspaceId: id, revision: 0, preferences: { theme: 'system', textScale: id === 'a' ? 100 : 150 } }, recovery: { missingAssets: 0, corruptAssets: 0, unsafePaths: 0, stagingFiles: 0, orphanAssets: 0 } }) } }];
  }));
  const factory = vi.fn((id: string) => ports.get(id)!);
  return { port, ports, factory };
}
it('retains unsaved forms, pending exact retries and delayed results inside their originating mounted course', async () => {
  const { port, ports, factory } = fixtures(); const a = ports.get('a')!; const b = ports.get('b')!;
  let finish!: (value: Library) => void;
  a.content.applyContent.mockReturnValueOnce(new Promise(resolve => { finish = resolve; }));
  render(<CourseApp coursePort={port} makePorts={factory} />); const user = userEvent.setup();
  await user.type(await screen.findByRole('textbox', { name: 'New subject' }), 'Draft A');
  await user.type(screen.getByRole('textbox', { name: 'New material' }), 'Unsaved material A');
  await user.click(screen.getByRole('button', { name: 'Create subject' }));
  await user.click(screen.getByRole('button', { name: 'Course B' }));
  await waitFor(() => expect(screen.getByRole('textbox', { name: 'New subject' })).toHaveValue(''));
  expect(screen.getByRole('textbox', { name: 'New material' })).toHaveValue('');
  await user.type(screen.getByRole('textbox', { name: 'New subject' }), 'Unsaved B');
  await act(async () => { const id = a.content.applyContent.mock.calls[0][0].command.id; finish({ ...blank('a'), revision: 1, subjects: { [id]: { id, title: 'Saved in A', archived: false, priorKnowledge: null } } }); });
  expect(screen.queryByRole('button', { name: 'Saved in A' })).not.toBeInTheDocument();
  expect(b.content.applyContent).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: '<script>A</script>' }));
  expect(await screen.findByRole('button', { name: 'Saved in A' })).toBeVisible();
  expect(screen.getByRole('textbox', { name: 'New material' })).toHaveValue('Unsaved material A');
  expect(document.documentElement.style.fontSize).toBe('100%');
  a.content.applyContent.mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementationOnce(async request => { const oldId = a.content.applyContent.mock.calls[0][0].command.id; return { ...blank('a'), revision: 2, subjects: { [oldId]: { id: oldId, title: 'Saved in A', archived: false, priorKnowledge: null }, [request.command.id]: { id: request.command.id, title: request.command.title, archived: false, priorKnowledge: null } } }; });
  await user.type(screen.getByRole('textbox', { name: 'New subject' }), 'Retry in A');
  await user.click(screen.getByRole('button', { name: 'Create subject' }));
  await screen.findByRole('button', { name: 'Retry last change' });
  await user.click(screen.getByRole('button', { name: 'Course B' }));
  expect(screen.getByRole('textbox', { name: 'New subject' })).toHaveValue('Unsaved B');
  expect(screen.queryByRole('button', { name: 'Retry last change' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: '<script>A</script>' }));
  await user.click(screen.getByRole('button', { name: 'Retry last change' }));
  expect(a.content.applyContent.mock.calls[2][0]).toEqual(a.content.applyContent.mock.calls[1][0]);
  expect(factory.mock.calls).toEqual([['a'], ['b']]);
  expect(document.querySelector('script')).toBeNull();
});
it.each([{ code: 'outcome_unconfirmed' }, new Error('Lost native response')])('retries an uncertain creation with the exact identity/title and opens the persisted selection (%j)', async reason => {
  const { port, factory } = fixtures(); vi.mocked(port.list).mockResolvedValue(catalogue('b'));
  vi.mocked(port.create).mockRejectedValueOnce(reason).mockResolvedValueOnce(catalogue('a'));
  render(<CourseApp coursePort={port} makePorts={factory} />); const user = userEvent.setup();
  await waitFor(() => expect(factory).toHaveBeenCalledWith('b'));
  await user.type(screen.getByLabelText('New course'), 'New course title');
  await user.click(screen.getByRole('button', { name: 'Create course' }));
  await user.click(await screen.findByRole('button', { name: 'Retry course action' }));
  expect(vi.mocked(port.create).mock.calls[1]).toEqual(vi.mocked(port.create).mock.calls[0]);
  expect(await screen.findByRole('button', { name: '<script>A</script>' })).toHaveAttribute('aria-current', 'page');
});

it('keeps a delayed file selection and unsaved source metadata in the originating course without moving focus', async () => {
  const { port, ports, factory } = fixtures(); const a = ports.get('a')!; const b = ports.get('b')!;
  const selected = { filename: 'original-a.txt', digest: 'a'.repeat(64), byteLength: 12 };
  let finish!: (value: typeof selected) => void;
  const pick = vi.fn().mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const save = vi.fn().mockImplementation(async request => ({ ...request.source, revision: 1, byteLength: 12 }));
  for (const [id, app] of [['a', a], ['b', b]] as const) {
    app.content.library.mockResolvedValue({ ...blank(id), resources: { r: { id: 'r', title: 'Material', archived: false } } });
    app.content.readDocument = vi.fn().mockResolvedValue(null);
    app.content.readingPosition = vi.fn().mockResolvedValue(null);
    app.content.annotations = vi.fn().mockResolvedValue([]);
    app.sources = { ...app.sources, sources: vi.fn().mockResolvedValue({ revision: 0, sources: [] }), pickSource: id === 'a' ? pick : vi.fn().mockResolvedValue(null), saveSource: save };
  }
  render(<CourseApp coursePort={port} makePorts={factory} />); const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Material' }));
  await user.click(await screen.findByRole('button', { name: 'Choose source file' }));
  await user.click(screen.getByRole('button', { name: 'Course B' }));
  await user.click(await screen.findByRole('button', { name: 'Material' }));
  const foreground = screen.getByRole('button', { name: 'Course B' }); foreground.focus();
  await act(async () => { finish(selected); });
  expect(foreground).toHaveFocus();
  expect(screen.queryByRole('button', { name: 'Attach selected source' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: '<script>A</script>' }));
  await user.type(await screen.findByRole('textbox', { name: 'Attribution' }), 'Notes for A');
  await user.click(screen.getByRole('button', { name: 'Course B' }));
  expect(screen.queryByRole('textbox', { name: 'Attribution' })).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: '<script>A</script>' }));
  expect(screen.getByRole('textbox', { name: 'Attribution' })).toHaveValue('Notes for A');
  await user.click(screen.getByRole('button', { name: 'Attach selected source' }));
  await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({ workspaceId: 'a', source: expect.objectContaining({ filename: 'original-a.txt', attribution: 'Notes for A' }) })));
  expect(pick).toHaveBeenCalledWith(expect.objectContaining({ workspaceId: 'a' }));
});

it('keeps existing courses and title corrections usable after a confirmed course-cap rejection', async () => {
  const { port, factory } = fixtures();
  const full: CourseCatalogue = { ...catalogue(), courses: [...catalogue().courses, ...Array.from({ length: 30 }, (_, index) => ({ id: `other-${index}`, title: `Other course ${index}` }))] };
  vi.mocked(port.list).mockResolvedValue(full);
  vi.mocked(port.select).mockImplementation(async id => ({ ...full, selectedCourseId: id }));
  vi.mocked(port.create).mockRejectedValue({ code: 'budget_exceeded' });
  render(<CourseApp coursePort={port} makePorts={factory} />); const user = userEvent.setup();
  await user.type(await screen.findByRole('textbox', { name: 'New course' }), 'Overflow');
  await user.click(screen.getByRole('button', { name: 'Create course' }));
  await screen.findByRole('alert');
  expect(screen.queryByRole('button', { name: 'Retry course action' })).not.toBeInTheDocument();
  const title = screen.getByRole('textbox', { name: 'New course' }); expect(title).toBeEnabled();
  await user.clear(title); await user.type(title, 'Corrected title');
  await user.click(screen.getByRole('button', { name: 'Course B' }));
  await waitFor(() => expect(port.select).toHaveBeenCalledWith('b'));
  expect(screen.getByRole('button', { name: 'Course B' })).toHaveAttribute('aria-current', 'page');
  expect(title).toHaveValue('Corrected title');
});

it('allows correcting a definitively invalid course title without retaining a doomed create action', async () => {
  const { port, factory } = fixtures(); vi.mocked(port.create).mockRejectedValueOnce({ code: 'invalid_request' }).mockResolvedValueOnce(catalogue('a'));
  render(<CourseApp coursePort={port} makePorts={factory} />); const user = userEvent.setup();
  await user.type(await screen.findByRole('textbox', { name: 'New course' }), 'Invalid title');
  await user.click(screen.getByRole('button', { name: 'Create course' })); await screen.findByRole('alert');
  const input = screen.getByRole('textbox', { name: 'New course' }); expect(input).toBeEnabled();
  await user.clear(input); await user.type(input, 'Corrected'); await user.click(screen.getByRole('button', { name: 'Create course' }));
  await waitFor(() => expect(port.create).toHaveBeenCalledTimes(2));
  expect(vi.mocked(port.create).mock.calls[1][0]).not.toBe(vi.mocked(port.create).mock.calls[0][0]);
  expect(vi.mocked(port.create).mock.calls[1][1]).toBe('Corrected');
});
