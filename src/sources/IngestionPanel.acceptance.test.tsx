import { expect, it, vi } from 'vitest';
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { IngestionPanel } from './IngestionPanel';
import type { IngestionJob, IngestionPort, JobSummary } from '../application/ingestionTypes';

const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.csv', mediaType: 'text/csv', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 20 };
const mapping = { keyColumn: 0, titleColumn: 1, bodyColumn: 2 };
const job: IngestionJob = { id: 'new', handlerVersion: 1, revision: 0, status: 'staged', previousJobId: null, source, mapping, outcomes: [], extraction: { contractVersion: 1, format: 'csv', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'csvRecord', value: '2' }, fields: ['key', 'Source title', 'Source body'] }] } };
const old: IngestionJob = { ...job, id: 'old', revision: 1, outcomes: [{ index: 0, key: 'key', resourceId: 'material', documentRevisionId: 'old-rev' }] };
const summary = (id: string): JobSummary => ({ id, sourceId: 's', revision: 1, status: 'staged', recordCount: 1, appliedCount: 1, eligibleBaseline: true });
const comparison = { jobId: 'new', previousJobId: 'old', items: [{ index: 0, key: 'key', resourceId: 'material', status: 'conflict', currentRevisionId: 'human-rev', currentTitle: 'Human title', currentBody: 'Human body', prior: { title: 'Prior title', body: 'Prior body', locator: { kind: 'csvRecord', value: '1' } }, incoming: { title: 'Source title', body: 'Source body', locator: { kind: 'csvRecord', value: '2' } } }] };
function portWith(overrides: Partial<IngestionPort> = {}): IngestionPort {
  return { list: vi.fn().mockResolvedValue([summary('old')]), create: vi.fn().mockResolvedValue(job), read: vi.fn().mockImplementation(async id => id === 'old' ? old : job), reimportPreview: vi.fn().mockResolvedValue(comparison), reimportApply: vi.fn().mockResolvedValue({ ...job, revision: 1 }), ...overrides } as IngestionPort;
}
function panel(port: IngestionPort, revision = 1, workspaceId = 'w') { return <IngestionPanel port={port} sources={[source]} workspaceId={workspaceId} resourceId="r" libraryRevision={revision} onChanged={vi.fn()} />; }
async function compare(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  await user.selectOptions(screen.getByLabelText('Reimport from'), 'old');
  await user.click(screen.getByRole('button', { name: 'Compare reimport' }));
}

it('loads later job pages and retains reachable baselines after a saved operation', async () => {
  const first = Array.from({ length: 20 }, (_, i) => summary(`j${i}`));
  const list = vi.fn().mockImplementation(async (_resource, offset) => offset === 0 ? first : [summary('old')]);
  render(panel(portWith({ list })));
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Load more import jobs' }));
  expect(await screen.findByRole('button', { name: 'Open import old' })).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  expect(await screen.findByRole('option', { name: 'old' })).toBeInTheDocument();
  expect(list).toHaveBeenCalledWith('r', 20);
  expect(screen.getByRole('button', { name: 'Load more import jobs' })).toBeDisabled();
});

it('shows prior, human and source values with locators and revokes approvals when content changes', async () => {
  const port = portWith(); const user = userEvent.setup();
  const view = render(panel(port)); await compare(user);
  expect(within(screen.getByRole('group', { name: 'Prior imported values for key' })).getByText('Prior body')).toBeVisible();
  expect(within(screen.getByRole('group', { name: 'Current human values for key' })).getByText('Human body')).toBeVisible();
  expect(within(screen.getByRole('group', { name: 'New source values for key' })).getByText('Source body')).toBeVisible();
  expect(screen.getByText('csvRecord: 1')).toBeVisible(); expect(screen.getByText('csvRecord: 2')).toBeVisible();
  const approval = screen.getByLabelText(/Explicitly replace current human edits/);
  approval.focus(); await user.keyboard(' ');
  const select = screen.getByLabelText('Select record 1'); select.focus(); await user.keyboard(' ');
  expect(screen.getByRole('button', { name: 'Apply selected records' })).toBeEnabled();
  view.rerender(panel(port, 2));
  expect(screen.queryByLabelText(/Explicitly replace current human edits/)).not.toBeInTheDocument();
  expect(screen.getByLabelText('Select record 1')).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Apply selected records' })).toBeDisabled();
});

it('discards old comparison approvals when a fresh comparison fails', async () => {
  const preview = vi.fn().mockResolvedValueOnce(comparison).mockRejectedValueOnce({ code: 'revision_conflict' });
  render(panel(portWith({ reimportPreview: preview }))); const user = userEvent.setup(); await compare(user);
  await user.click(screen.getByLabelText(/Explicitly replace current human edits/));
  await user.click(screen.getByLabelText('Select record 1'));
  await user.click(screen.getByRole('button', { name: 'Compare reimport' }));
  expect(await screen.findByRole('alert')).toBeVisible();
  expect(screen.queryByLabelText(/Explicitly replace current human edits/)).not.toBeInTheDocument();
  expect(screen.getByLabelText('Select record 1')).toBeDisabled();
});

it('clears locks and stale status on course switch and ignores late completion', async () => {
  let resolve!: (value: IngestionJob) => void;
  const pending = new Promise<IngestionJob>(done => { resolve = done; });
  const original = portWith({ create: vi.fn().mockReturnValue(pending) });
  const view = render(panel(original)); const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  const next = portWith({ list: vi.fn().mockResolvedValue([]) });
  view.rerender(panel(next, 1, 'w'));
  expect(await screen.findByRole('button', { name: 'Create import preview' })).toBeEnabled();
  await act(async () => resolve(job));
  expect(screen.queryByText('Import rows.csv')).not.toBeInTheDocument();
  expect(screen.queryByRole('status')).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  expect(next.create).toHaveBeenCalledWith(expect.objectContaining({ workspaceId: 'w' }));
});

it.each(['cancel', 'review', 'reimport', 'undo'] as const)('retries an unconfirmed %s with the identical envelope and shows a confirmed rejection', async kind => {
  const mutation = vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockRejectedValueOnce({ code: 'revision_conflict' });
  const port = portWith({ cancel: mutation, review: mutation, reimportApply: mutation, undo: mutation, undoPreview: vi.fn().mockResolvedValue({ jobId: 'new', canUndo: true, resources: ['material'], diagnostics: [] }) });
  render(panel(port)); const user = userEvent.setup();
  if (kind === 'reimport') { await compare(user); await user.click(screen.getByLabelText(/Explicitly replace current human edits/)); await user.click(screen.getByLabelText('Select record 1')); await user.click(screen.getByRole('button', { name: 'Apply selected records' })); }
  else { await user.click(screen.getByRole('button', { name: 'Create import preview' }));
    if (kind === 'cancel') await user.click(screen.getByRole('button', { name: 'Cancel remaining import' }));
    if (kind === 'review') await user.click(screen.getByRole('button', { name: 'Exclude record 1' }));
    if (kind === 'undo') { await user.click(screen.getByRole('button', { name: 'Preview undo' })); await user.click(screen.getByRole('button', { name: 'Confirm undo' })); }
  }
  const first = structuredClone(mutation.mock.calls[0][0]);
  expect(screen.getByRole('button', { name: 'Create import preview' })).toBeDisabled();
  await user.click(await screen.findByRole('button', { name: 'Retry same import' }));
  expect(mutation.mock.calls[1][0]).toEqual(first);
  expect(await screen.findByRole('alert')).toBeVisible();
  expect(screen.queryByRole('button', { name: 'Retry same import' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Confirm undo' })).not.toBeInTheDocument();
});

it('uses the first saved source when sources arrive after mount and can reopen a lost-reply job', async () => {
  const create = vi.fn().mockRejectedValue({ code: 'outcome_unconfirmed' });
  const read = vi.fn().mockResolvedValue({ ...job, revision: 1, outcomes: [{ index: 0, key: 'key', resourceId: 'material', documentRevisionId: 'v' }] });
  const port = portWith({ create, read, list: vi.fn().mockResolvedValue([summary('new')]) });
  const view = render(<IngestionPanel port={port} sources={[]} workspaceId="w" resourceId="r" libraryRevision={1} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  view.rerender(panel(port));
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  expect(create).toHaveBeenCalledWith(expect.objectContaining({ sourceId: 's', sourceRevision: 1 }));
  expect(await screen.findByRole('button', { name: 'Retry same import' })).toBeVisible();
  view.unmount();
  render(panel(port));
  await user.click(await screen.findByRole('button', { name: 'Open import new' }));
  expect(await screen.findByLabelText('Select record 1')).toBeDisabled();
  expect(screen.queryByRole('button', { name: 'Retry same import' })).not.toBeInTheDocument();
  expect(read).toHaveBeenCalledWith('new');
});

it('ignores delayed old-course list errors and comparison responses', async () => {
  let resolve!: (value: unknown) => void; let reject!: (value: unknown) => void;
  const firstList = Promise.resolve([summary('old')]);
  const laterList = new Promise<JobSummary[]>((_, fail) => { reject = fail; });
  const preview = new Promise(done => { resolve = done; });
  const list = vi.fn().mockReturnValueOnce(firstList).mockReturnValue(laterList);
  const port = portWith({ list, reimportPreview: vi.fn().mockReturnValue(preview) });
  const view = render(panel(port)); const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  await user.selectOptions(screen.getByLabelText('Reimport from'), 'old');
  await user.click(screen.getByRole('button', { name: 'Compare reimport' }));
  const next = portWith({ list: vi.fn().mockResolvedValue([]) });
  view.rerender(panel(next, 1, 'next'));
  await act(async () => { reject({ code: 'invalid_request' }); resolve(comparison); });
  expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  expect(screen.queryByLabelText(/Explicitly replace current human edits/)).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Open import old' })).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Create import preview' })).toBeEnabled();
});

it('keeps duplicate and blank CSV keys unresolved and labels every remaining item outcome', async () => {
  const records = [['duplicate', 'One', 'body'], ['duplicate', 'Two', 'body'], ['', '', ''], ['valid', 'Valid', 'body']].map((fields, index) => ({ fields, locator: { kind: 'csvRecord' as const, value: String(index + 1) } }));
  const ambiguous = { ...job, extraction: { ...job.extraction, records } };
  const review = vi.fn().mockImplementation(async request => ({ ...ambiguous, revision: 1, decisions: request.decisions }));
  render(panel(portWith({ create: vi.fn().mockResolvedValue(ambiguous), review })));
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  expect(screen.getByLabelText('Select record 1')).toBeDisabled();
  expect(screen.getByLabelText('Select record 2')).toBeDisabled();
  expect(screen.getByLabelText('Select record 3')).toBeDisabled();
  expect(screen.getByLabelText('Select record 4')).toBeEnabled();
  expect(screen.getByText('Outcomes: 0 applied, 0 reviewed, 4 pending.')).toBeVisible();
  await user.click(screen.getByRole('button', { name: 'Exclude record 1' }));
  expect(review).toHaveBeenCalledWith(expect.objectContaining({ decisions: [{ index: 0, key: null, disposition: 'unresolved' }] }));
  expect(await screen.findByText('Outcomes: 0 applied, 1 reviewed, 3 pending.')).toBeVisible();
});
