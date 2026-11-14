import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { createNativeLessonPort } from '../lessons/nativeLessonPort';
import { ReviewDashboard } from './ReviewDashboard';
import type { ReviewOverview, ReviewPort } from './reviewTypes';
import type { StudyPort } from './types';

function fixture() {
  const view: ReviewOverview = { plan: { contractVersion: 1, revision: 0, generation: 1, timeZone: 'Etc/UTC', dayStartHour: 4, dailyNewLimit: 10, dailyReviewLimit: 50, retentionBasisPoints: 9000, reminders: [] }, nowUnixMs: 1, today: 20000, todayDate: '2026-11-14', timeZoneRules: 'pinned', clockRollback: false, newUsed: 0, reviewsUsed: 0, dueTotal: 0, newTotal: 0, offered: [] };
  const port: ReviewPort = { overview: vi.fn(async () => view), apply: vi.fn(async () => view.plan), recover: vi.fn(async () => undefined) };
  const study: StudyPort = { sessions: vi.fn(), read: vi.fn(), apply: vi.fn(), recover: vi.fn() };
  return { view, port, study };
}
it('retries an uncertain review policy write with the same operation and expected revision', async () => {
  const { port, study } = fixture(); vi.mocked(port.apply).mockRejectedValueOnce({ code: 'outcome_unconfirmed' });
  render(<ReviewDashboard port={port} study={study} lessons={createNativeLessonPort('course')} units={{}} />);
  fireEvent.change(await screen.findByLabelText('Daily new recall limit'), { target: { value: '0' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save recall policy' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Reconnect and retry review change' }));
  await waitFor(() => expect(port.apply).toHaveBeenCalledTimes(2));
  expect(vi.mocked(port.apply).mock.calls[1][0]).toEqual(vi.mocked(port.apply).mock.calls[0][0]);
  expect(vi.mocked(port.apply).mock.calls[0][0].action).toMatchObject({ kind: 'setPolicy', dailyNewLimit: 0 });
  expect(port.recover).toHaveBeenCalledOnce(); expect(study.apply).not.toHaveBeenCalled();
});
it('completes a practical reminder without fabricating a study attempt', async () => {
  const { view, port, study } = fixture();
  view.plan.reminders = [{ id: 'reminder', unitId: 'unit', activityRevisionId: 'activity', dueDay: 19999, intervalDays: 7, archived: false, completedAtUnixMs: null }];
  render(<ReviewDashboard port={port} study={study} lessons={createNativeLessonPort('course')} units={{ unit: { title: 'Cooking', archived: false } }} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Mark reminder completed' }));
  await waitFor(() => expect(port.apply).toHaveBeenCalledOnce());
  expect(vi.mocked(port.apply).mock.calls[0][0].action).toEqual({ kind: 'completeReminder', id: 'reminder' }); expect(study.apply).not.toHaveBeenCalled();
});
