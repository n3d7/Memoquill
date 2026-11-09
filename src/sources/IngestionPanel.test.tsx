import { expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { IngestionPanel } from './IngestionPanel';
import type { IngestionJob, IngestionPort } from '../application/ingestionTypes';

it('reviews a selected subset and retries an unconfirmed apply with the same operation', async () => {
  const source = { id: 's', resourceId: 'r', digest: 'digest', filename: 'rows.csv', mediaType: 'text/csv', attribution: null, license: null, originUrl: null, revision: 1, byteLength: 8 };
  const job: IngestionJob = { id: 'j', handlerVersion: 1, revision: 0, status: 'staged', previousJobId: null, source, mapping: null, outcomes: [], extraction: { contractVersion: 1, format: 'csv', status: 'complete', diagnostic: null, records: [{ locator: { kind: 'csvRecord', value: '1' }, fields: ['one', 'body'] }, { locator: { kind: 'csvRecord', value: '2' }, fields: ['two', 'text'] }] } };
  const apply = vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockResolvedValue({ ...job, revision: 1, outcomes: [{ index: 1, key: 'two', resourceId: 'new', documentRevisionId: 'v' }] });
  const port = { list: vi.fn().mockResolvedValue([]), create: vi.fn().mockResolvedValue(job), apply } as unknown as IngestionPort;
  const reload = vi.fn();
  render(<IngestionPanel port={port} sources={[source]} workspaceId="w" resourceId="r" libraryRevision={2} onChanged={reload} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Create import preview' }));
  await user.clear(screen.getByLabelText('Body column')); await user.type(screen.getByLabelText('Body column'), '2');
  await user.click(await screen.findByLabelText('Select record 2'));
  await user.click(screen.getByRole('button', { name: 'Apply selected records' }));
  await user.click(await screen.findByRole('button', { name: 'Retry same import' }));
  expect(apply.mock.calls[0][0]).toEqual(apply.mock.calls[1][0]);
  expect(apply.mock.calls[0][0].selected).toEqual([1]);
  expect(reload).toHaveBeenCalled();
});
