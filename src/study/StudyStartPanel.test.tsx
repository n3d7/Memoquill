import { ApplicationProvider } from '../application/ApplicationProvider';
import { LessonTabs } from '../lessons/LessonTabs';
import type { Library } from '../application/contentTypes';
import type { SourcePort } from '../application/ports';
import { StudySettingsProvider } from './sessionSettings';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { createNativeLessonPort } from '../lessons/nativeLessonPort';
import { StudyStartPanel } from './StudyStartPanel';
import type { ContinuityPort, ResumePage } from './continuityTypes';
import type { ReviewOverview, ReviewPort } from './reviewTypes';
import type { StudyMutation, StudyPort, StudySession } from './types';

vi.mock('./StudyPanel', () => ({ StudyPanel: ({ initialSessionId, onSafeToLeave }: { initialSessionId: string; onSafeToLeave: (value: boolean) => void }) => <section aria-label="Resumed study"><p>Active session: {initialSessionId}</p><button onClick={() => onSafeToLeave(true)}>Acknowledge saved study state</button></section> }));
function session(): StudySession {
  const item = { bindingId: 'binding', activity: { id: 'activity', activityId: 'activity', unitId: 'unit', definition: { family: 'recall' as const, version: 1 as const, prompt: 'Question', answer: 'Answer', hints: [], evaluation: 'exact' as const } } };
  return { contractVersion: 1, id: 'saved', unitId: 'unit', compositionRevisionId: 'composition', mode: 'practice', phase: 'independent', revision: 2, cursor: 0, items: [item], draft: { response: { kind: 'text', value: 'Saved response' }, help: { hintsRevealed: 0, solutionRevealed: false, external: 'unknown' } } };
}
function fixture() {
  const sessions: ResumePage = { contractVersion: 1, offset: 0, nextOffset: null, sessions: [{ sessionId: 'saved', unitId: 'unit', title: 'English', compositionRevisionId: 'composition', cursor: 0, total: 1, mode: 'practice', phase: 'independent', savedResponse: true, changedContent: true }] };
  const view: ReviewOverview = { plan: { contractVersion: 1, revision: 0, generation: 1, timeZone: 'Etc/UTC', dayStartHour: 4, dailyNewLimit: 10, dailyReviewLimit: 50, retentionBasisPoints: 9000, reminders: [] }, nowUnixMs: 1, today: 20000, todayDate: '2026-11-14', timeZoneRules: 'pinned', clockRollback: false, newUsed: 0, reviewsUsed: 0, dueTotal: 1, newTotal: 0, offered: [{ unitId: 'unit', compositionRevisionId: 'composition', bindingId: 'binding', activityRevisionId: 'activity', prompt: 'Question', dueDay: 20000, dueDate: '2026-11-14' }] };
  const continuity: ContinuityPort = { sessions: vi.fn(async () => sessions) };
  const review: ReviewPort = { overview: vi.fn(async () => view), apply: vi.fn(), recover: vi.fn() };
  const study: StudyPort = { sessions: vi.fn(), read: vi.fn(async () => ({ session: session(), attempts: [], judgments: [], changedContent: true })), apply: vi.fn(async (request: StudyMutation) => ({ ...session(), id: request.sessionId })), recover: vi.fn(async () => undefined) };
  return { sessions, view, continuity, review, study, lessons: createNativeLessonPort('course') };
}

it('resumes an acknowledged session in one action without creating attempts or replacing its revision', async () => {
  const f = fixture(); render(<StudyStartPanel {...f} />);
  const resume = await screen.findByRole('button', { name: 'Resume saved session' });
  await waitFor(() => expect(resume).toBeEnabled());
  expect(f.study.apply).not.toHaveBeenCalled(); fireEvent.click(resume);
  await screen.findByRole('region', { name: 'Resumed study' });
  expect(f.study.read).toHaveBeenCalledWith('saved'); expect(f.study.apply).not.toHaveBeenCalled();
  expect(screen.getByText(/Content changed; resume keeps the pinned/)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Return to start actions' })).toBeDisabled();
  fireEvent.click(screen.getByRole('button', { name: 'Acknowledge saved study state' }));
  expect(screen.getByRole('button', { name: 'Return to start actions' })).toBeEnabled();
});

it('starts only an offered due recall and retains the operation on uncertain retry', async () => {
  const f = fixture(); vi.mocked(f.study.apply).mockRejectedValueOnce({ code: 'OutcomeUnconfirmed' });
  render(<StudyStartPanel {...f} />); const due = screen.getByRole('button', { name: 'Review due recall' });
  await waitFor(() => expect(due).toBeEnabled()); fireEvent.click(due);
  fireEvent.click(await screen.findByRole('button', { name: 'Reconnect and retry due session' }));
  await screen.findByRole('region', { name: 'Resumed study' });
  expect(f.study.apply).toHaveBeenCalledTimes(2);
  expect(vi.mocked(f.study.apply).mock.calls[1][0]).toEqual(vi.mocked(f.study.apply).mock.calls[0][0]);
  expect(vi.mocked(f.study.apply).mock.calls[0][0].action).toMatchObject({ kind: 'start', bindingIds: ['binding'], compositionRevisionId: 'composition' });
  expect(f.study.recover).toHaveBeenCalledOnce(); expect(f.review.apply).not.toHaveBeenCalled();
});

it.each(['new-only', 'daily-limit', 'clock-rollback'])('does not bypass policy when due recall is unavailable: %s', async reason => {
  const f = fixture(); if (reason === 'new-only') f.view.offered[0].dueDay = null;
  if (reason === 'daily-limit') f.view.offered = [];
  if (reason === 'clock-rollback') f.view.clockRollback = true;
  render(<StudyStartPanel {...f} />); await screen.findByText(/1 due recall items/);
  expect(screen.getByRole('button', { name: 'Review due recall' })).toBeDisabled();
  expect(f.study.apply).not.toHaveBeenCalled();
});

it('paginates older saved sessions and restores focus without starting them', async () => {
  const f = fixture(); f.sessions.nextOffset = 64;
  vi.mocked(f.continuity.sessions).mockResolvedValueOnce(f.sessions).mockResolvedValueOnce({ ...f.sessions, offset: 64, nextOffset: null });
  render(<StudyStartPanel {...f} />); const older = await screen.findByRole('button', { name: 'Older saved sessions' });
  await waitFor(() => expect(older).toBeEnabled()); fireEvent.click(older);
  await waitFor(() => expect(screen.getByRole('heading', { name: 'Saved unfinished sessions' })).toHaveFocus());
  expect(f.continuity.sessions).toHaveBeenLastCalledWith(64);
  expect(screen.getByRole('button', { name: 'Resume saved session' })).toBeDisabled();
  expect(f.study.read).not.toHaveBeenCalled();
});

it('rejects a changed or foreign session response', async () => {
  const f = fixture(); vi.mocked(f.study.read).mockResolvedValueOnce({ session: { ...session(), unitId: 'foreign' }, attempts: [], judgments: [], changedContent: true });
  render(<StudyStartPanel {...f} />); const resume = screen.getByRole('button', { name: 'Resume saved session' });
  await waitFor(() => expect(resume).toBeEnabled()); fireEvent.click(resume); await screen.findByRole('alert');
  expect(screen.queryByRole('region', { name: 'Resumed study' })).not.toBeInTheDocument();
});

it('does not show an old course overview after its ports change', async () => {
  const f = fixture(); let resolve: (value: ResumePage) => void = () => { throw new Error('No pending request'); };
  vi.mocked(f.continuity.sessions).mockImplementation(() => new Promise(value => { resolve = value; }));
  const { rerender } = render(<StudyStartPanel {...f} />);
  const next = fixture(); next.sessions.sessions = [];
  rerender(<StudyStartPanel {...next} />); await screen.findByText('No unfinished saved session on this page.');
  await act(async () => { resolve(f.sessions); });
  expect(screen.getByRole('button', { name: 'Resume saved session' })).toBeDisabled();
  expect(screen.queryByRole('button', { name: 'Resume English' })).not.toBeInTheDocument();
});

it('snapshots loaded settings for Due while Resume remains available without defaults', async () => {
  const f = fixture(); const settings = { theme: 'dark' as const, textScale: 125 as const, goal: 'Recall', focus: 'Today' };
  const { rerender } = render(<StudySettingsProvider settings={null}><StudyStartPanel {...f} /></StudySettingsProvider>);
  await screen.findByText(/1 due recall items/);
  expect(screen.getByRole('button', { name: 'Review due recall' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Resume saved session' })).toBeEnabled();
  rerender(<StudySettingsProvider settings={settings}><StudyStartPanel {...f} /></StudySettingsProvider>);
  fireEvent.click(screen.getByRole('button', { name: 'Review due recall' }));
  await waitFor(() => expect(f.study.apply).toHaveBeenCalledOnce());
  expect(vi.mocked(f.study.apply).mock.calls[0][0].action).toMatchObject({ kind: 'start', settings: { contractVersion: 1, ...settings } });
});

it('shares the course opening overview and reads the port again on refresh', async () => {
  const { continuity, review, study, lessons } = fixture();
  const data: Library = {
    workspaceId: 'workspace', revision: 0, subjects: {}, goals: {}, collections: {},
    resources: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {},
  };
  render(<ApplicationProvider ports={{ continuity, review, study, lessons }}>
    <LessonTabs data={data} port={lessons} sources={{} as SourcePort} perform={vi.fn()} />
  </ApplicationProvider>);

  expect(await screen.findByText(/1 due recall items/)).toBeVisible();
  expect(await screen.findByRole('button', { name: 'Refresh recall queue' })).toBeVisible();
  expect(review.overview).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole('button', { name: 'Refresh start actions' }));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Resume saved session' })).toBeEnabled());
  expect(review.overview).toHaveBeenCalledTimes(2);
});
