import { expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { IngestionPanel } from './IngestionPanel';
import type { IngestionJob, IngestionPort } from '../application/ingestionTypes';

it('reviews a selected subset and retries an unconfirmed apply with the same operation', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.csv', mediaType: 'text/csv', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 8 };
  const job: IngestionJob = { id: 'j', handlerVersion: 1, revision: 0, status: 'staged', previousJobId: null, source, mapping: null, outcomes: [], extraction: { contractVersion: 1, format: 'csv', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'csvRecord', value: '1' }, fields: ['one', 'body'] }, { locator: { kind: 'csvRecord', value: '2' }, fields: ['two', 'text'] }] } };
  const apply = vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockResolvedValue({ ...job, revision: 1, outcomes: [{ index: 1, key: 'two', resourceId: 'new', documentRevisionId: 'v' }] });
  const review = vi.fn().mockImplementation(async request => ({ ...job, revision: 2, outcomes: [{ index: 1, key: 'two', resourceId: 'new', documentRevisionId: 'v' }], decisions: request.decisions }));
  const port = { list: vi.fn().mockResolvedValue([]), create: vi.fn().mockResolvedValue(job), apply, review } as unknown as IngestionPort;
  const reload = vi.fn();
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={2} onChanged={reload} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  fireEvent.change(screen.getByLabelText('Body column'), { target: { value: '2' } });
  await user.click(await screen.findByLabelText('Select record 2'));
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  await user.click(await screen.findByRole('button', { name: 'Retry same import' }));
  expect(apply.mock.calls[0][0]).toEqual(apply.mock.calls[1][0]);
  expect(apply.mock.calls[0][0].selected).toEqual([1]);
  expect(reload).toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Exclude record 1' }));
  expect(review).toHaveBeenCalledWith(expect.objectContaining({ decisions: [{ index: 0, key: null, disposition: 'omitted' }] }));
  expect(await screen.findByRole('button', { name: 'Reopen record 1' })).toBeVisible();
});

it('requires explicit approval before sending an overwrite of human edits', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.csv', mediaType: 'text/csv', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 8 };
  const job: IngestionJob = { id: 'j', handlerVersion: 1, revision: 0, status: 'staged', previousJobId: null, source, mapping: null, outcomes: [], extraction: { contractVersion: 1, format: 'csv', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'csvRecord', value: '1' }, fields: ['one'] }] } };
  const apply = vi.fn().mockResolvedValue({ ...job, revision: 1 });
  const port = { list: vi.fn().mockResolvedValue([{ id: 'old', sourceId: 's', revision: 2, status: 'staged', recordCount: 1, appliedCount: 1, eligibleBaseline: true }]), create: vi.fn().mockResolvedValue(job), read: vi.fn().mockResolvedValue({ ...job, id: 'old', revision: 2 }), reimportPreview: vi.fn().mockResolvedValue({ jobId: 'j', previousJobId: 'old', items: [{ index: 0, key: 'one', resourceId: 'x', status: 'conflict', currentRevisionId: 'human-rev', currentTitle: 'Human title' }] }), reimportApply: apply } as unknown as IngestionPort;
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={4} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  await user.selectOptions(await screen.findByLabelText('Reimport from'), 'old');
  await user.click(screen.getByRole('button', { name: 'Compare reimport' }));
  const select = await screen.findByLabelText('Select record 1'); expect(select).toBeDisabled();
  await user.click(screen.getByLabelText(/Explicitly replace current human edits/));
  await user.click(select);
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  expect(apply).toHaveBeenCalledWith(expect.objectContaining({ selected: [0], resolutions: [{ index: 0, expectedDocumentRevisionId: 'human-rev', expectedTitle: 'Human title' }] }));
});

it('continues a reviewed-only child lineage while keeping its parent protected', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.csv', mediaType: 'text/csv', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 8 };
  const mapping = { keyColumn: 0, titleColumn: 0, bodyColumn: 0 };
  const base: IngestionJob = { id: 'a', handlerVersion: 1, revision: 1, status: 'staged', previousJobId: null, source, mapping, outcomes: [{ index: 0, key: 'one', resourceId: 'x', documentRevisionId: 'v' }], extraction: { contractVersion: 1, format: 'csv', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'csvRecord', value: '1' }, fields: ['one'] }] } };
  const reviewed: IngestionJob = { ...base, id: 'b', previousJobId: 'a', outcomes: [], decisions: [{ index: null, key: 'old', disposition: 'removedRetained' }] };
  const next: IngestionJob = { ...base, id: 'c', revision: 0, previousJobId: null, mapping: null, outcomes: [] };
  const jobs = [base, reviewed, next];
  const summaries = [{ id: 'a', appliedCount: 1, eligibleBaseline: false }, { id: 'b', appliedCount: 0, eligibleBaseline: true }, { id: 'c', appliedCount: 0, eligibleBaseline: false }].map(item => ({ ...item, sourceId: 's', revision: item.id === 'c' ? 0 : 1, status: 'staged', recordCount: 1 }));
  const compare = vi.fn().mockResolvedValue({ jobId: 'c', previousJobId: 'b', items: [{ index: 0, key: 'one', resourceId: 'x', status: 'unchanged', currentRevisionId: null, currentTitle: null }] });
  const reimportApply = vi.fn().mockResolvedValue({ ...next, revision: 1, previousJobId: 'b' });
  const port = { list: vi.fn().mockResolvedValue(summaries), read: vi.fn(async (id: string) => jobs.find(job => job.id === id)), reimportPreview: compare, reimportApply } as unknown as IngestionPort;
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={1} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Open import c' }));
  const previous = await screen.findByLabelText('Reimport from');
  expect(screen.queryByRole('option', { name: 'a' })).not.toBeInTheDocument();
  await user.selectOptions(previous, 'b');
  await user.click(screen.getByRole('button', { name: 'Compare reimport' }));
  expect(compare).toHaveBeenCalledWith('c', 'b', mapping);
  await user.click(await screen.findByLabelText('Select record 1'));
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  expect(reimportApply).toHaveBeenCalledWith(expect.objectContaining({ previousJobId: 'b', expectedPreviousRevision: 1 }));
  await user.click(await screen.findByRole('button', { name: 'Open import b' }));
  expect(await screen.findByRole('option', { name: 'a' })).toBeInTheDocument();
  expect(screen.getByLabelText('Reimport from')).toHaveValue('a');
});


it('identifies JSON properties and original pointers and blocks ambiguous records', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.json', mediaType: 'application/json', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 80 };
  const job = { id: 'j', handlerVersion: 2, revision: 0, status: 'staged', previousJobId: null, source, mapping: null, outcomes: [], extraction: { contractVersion: 2, format: 'json', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'jsonPointer', value: '/0' }, fields: ['{"id":"κ","title":"Привіт","body":{"n":1}}'], properties: { id: 'κ', title: 'Привіт', body: '{"n":1}' } }, { locator: { kind: 'jsonPointer', value: '/1' }, fields: ['{"id":1,"id":2}'], duplicateProperties: ['id'] }] } } as IngestionJob;
  const apply = vi.fn().mockResolvedValue({ ...job, revision: 1 });
  const review = vi.fn().mockImplementation(async request => ({ ...job, revision: 1, mapping: request.mapping, decisions: request.decisions }));
  const port = { list: vi.fn().mockResolvedValue([]), create: vi.fn().mockResolvedValue(job), apply, review } as unknown as IngestionPort;
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={1} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  expect(await screen.findByLabelText('Key property')).toBeVisible();
  await user.selectOptions(screen.getByLabelText('Key property'), 'property:id');
  await user.selectOptions(screen.getByLabelText('Title property'), 'property:title');
  await user.selectOptions(screen.getByLabelText('Body property'), 'property:body');
  expect(screen.getByText('Original JSON pointer: /0')).toBeVisible();
  expect(screen.getByText('Selected values — key: κ; title: Привіт; body: {"n":1}')).toBeVisible();
  expect(screen.getByText('Duplicate JSON properties: id — unresolved')).toBeVisible();
  expect(screen.getByLabelText('Select record 2')).toBeDisabled();
  await user.click(screen.getByLabelText('Select record 1'));
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  expect(apply).toHaveBeenCalledWith(expect.objectContaining({ mapping: { keyColumn: 0, titleColumn: 0, bodyColumn: 0, jsonProperties: { key: 'id', title: 'title', body: 'body' } }, selected: [0] }));
  await user.click(screen.getByRole('button', { name: 'Exclude record 2' }));
  expect(review).toHaveBeenCalledWith(expect.objectContaining({ decisions: [{ index: 1, key: null, disposition: 'unresolved' }] }));
});


it('keeps persisted handler-one JSON jobs in their original raw numeric contract', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'old.json', mediaType: 'application/json', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 20 };
  const job: IngestionJob = { id: 'old', handlerVersion: 1, revision: 0, status: 'staged', previousJobId: null, source, mapping: null, outcomes: [], extraction: { contractVersion: 1, format: 'json', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'jsonPointer', value: '' }, fields: ['{"id":1,"id":2}'] }] } };
  const apply = vi.fn().mockResolvedValue({ ...job, revision: 1 });
  const port = { list: vi.fn().mockResolvedValue([{ id: 'old' }]), read: vi.fn().mockResolvedValue(job), apply } as unknown as IngestionPort;
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={1} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Open import old' }));
  expect(screen.queryByLabelText('Key property')).not.toBeInTheDocument();
  expect(screen.getByLabelText('Key column')).toHaveValue(1);
  await user.click(screen.getByLabelText('Select record 1'));
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  expect(apply).toHaveBeenCalledWith(expect.objectContaining({ mapping: { keyColumn: 0, titleColumn: 0, bodyColumn: 0 }, selected: [0] }));
});


it('explicitly continues a legacy raw baseline with a version-two JSON job', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.json', mediaType: 'application/json', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 12 };
  const mapping = { keyColumn: 0, titleColumn: 0, bodyColumn: 0 };
  const legacy: IngestionJob = { id: 'old', handlerVersion: 1, revision: 1, status: 'staged', previousJobId: null, source, mapping, outcomes: [{ index: 0, key: '{"id":"one"}', resourceId: 'material', documentRevisionId: 'rev' }], extraction: { contractVersion: 1, format: 'json', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'jsonPointer', value: '' }, fields: ['{"id":"one"}'] }] } };
  const job: IngestionJob = { ...legacy, id: 'new', handlerVersion: 2, revision: 0, mapping: null, outcomes: [], extraction: { ...legacy.extraction, contractVersion: 2, records: [{ ...legacy.extraction.records[0], properties: { id: 'one' } }] } };
  const compare = vi.fn().mockResolvedValue({ jobId: 'new', previousJobId: 'old', items: [{ index: 0, key: '{"id":"one"}', resourceId: 'material', status: 'unchanged', currentRevisionId: null, currentTitle: null }] });
  const reimportApply = vi.fn().mockResolvedValue({ ...job, revision: 1, mapping, previousJobId: 'old', outcomes: legacy.outcomes });
  const port = { list: vi.fn().mockResolvedValue([{ id: 'old', eligibleBaseline: true }]), create: vi.fn().mockResolvedValue(job), read: vi.fn().mockResolvedValue(legacy), reimportPreview: compare, reimportApply } as unknown as IngestionPort;
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={1} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  await user.click(screen.getByLabelText('Use legacy raw JSON mapping'));
  expect(screen.getByLabelText('Key property')).toHaveValue('raw:');
  expect(screen.getByText('Selected values — key: {"id":"one"}; title: {"id":"one"}; body: {"id":"one"}')).toBeVisible();
  await user.selectOptions(screen.getByLabelText('Reimport from'), 'old');
  await user.click(screen.getByRole('button', { name: 'Compare reimport' }));
  expect(compare).toHaveBeenCalledWith('new', 'old', mapping);
  await user.click(await screen.findByLabelText('Select record 1'));
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  expect(reimportApply).toHaveBeenCalledWith(expect.objectContaining({ mapping, selected: [0], previousJobId: 'old' }));
});


it('treats absent prototype-like property names as missing instead of inherited values', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.json', mediaType: 'application/json', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 50 };
  const job: IngestionJob = { id: 'j', handlerVersion: 2, revision: 0, status: 'staged', previousJobId: null, source, mapping: null, outcomes: [], extraction: { contractVersion: 2, format: 'json', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'jsonPointer', value: '/0' }, fields: ['{"constructor":"safe"}'], properties: { constructor: 'safe' } }, { locator: { kind: 'jsonPointer', value: '/1' }, fields: ['{"id":"two"}'], properties: { id: 'two' } }] } };
  const port = { list: vi.fn().mockResolvedValue([]), create: vi.fn().mockResolvedValue(job) } as unknown as IngestionPort;
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={1} onChanged={vi.fn()} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  await user.selectOptions(screen.getByLabelText('Key property'), 'property:constructor');
  expect(screen.getByLabelText('Select record 1')).toBeEnabled();
  expect(screen.getByLabelText('Select record 2')).toBeDisabled();
  expect(screen.getByText('Missing selected JSON property — unresolved')).toBeVisible();
});
