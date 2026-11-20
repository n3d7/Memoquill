import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { ProgressPanel } from './ProgressPanel';
import type { LessonProgress, ProgressEvidence, ProgressPort } from './progressTypes';

function evidence(id = 'attempt'): ProgressEvidence {
  return { attempt: { id, sessionId: 'session', itemIndex: 0, activityRevisionId: 'activity', response: { kind: 'text', value: 'My response' }, help: { hintsRevealed: 1, solutionRevealed: false, external: 'unknown' }, selfRating: 4, submittedAtUnixMs: null }, bindingId: 'binding', compositionRevisionId: 'composition', outcome: 'recall', mode: 'practice', phase: 'guided', currentActivity: true, changedComposition: false, feedbackAvailable: true, judgments: [], sources: [] };
}
function snapshot(offset = 0): LessonProgress {
  return { contractVersion: 1, unitId: 'unit', currentCompositionRevisionId: 'composition', sessionCount: 1, totalAttempts: 1, offset, nextOffset: null, coverage: [{ outcome: 'recall', currentActivityBindings: 2, evidencedCurrentActivityBindings: 1, attempts: 1, previousActivityAttempts: 0, feedbackWithheldAttempts: 0, unassessedAttempts: 1, judgments: [] }], evidence: [evidence()] };
}
function fixture(value = snapshot()) {
  const read = vi.fn(async () => value);
  return { read, port: { read } satisfies ProgressPort };
}
async function show() { fireEvent.click(screen.getByRole('button', { name: 'Show lesson progress' })); await screen.findByRole('list', { name: 'Outcome coverage' }); }

it('loads on demand and keeps assistance, confidence and missing evidence honest', async () => {
  const { port, read } = fixture(); render(<ProgressPanel unitId="unit" port={port} />);
  expect(read).not.toHaveBeenCalled(); await show();
  expect(read).toHaveBeenCalledWith('unit', 0);
  expect(screen.getByText(/1 of 2 current activity bindings/)).toBeInTheDocument();
  fireEvent.click(screen.getByText(/^Inspect attempt:/));
  expect(screen.getByText(/External assistance: Unknown; not verified unaided/)).toBeInTheDocument();
  expect(screen.getByText(/These ratings are not independent correctness/)).toBeInTheDocument();
  expect(screen.getByText(/Unassessed: no judgment recorded/)).toBeInTheDocument();
  expect(screen.getByText(/First saved: Not recorded/)).toBeInTheDocument();
  expect(screen.getByText(/Source references do not establish/)).toBeInTheDocument();
});

it('retains superseded and self-report judgments without pretending they verify competence', async () => {
  const p = snapshot();
  p.evidence[0].judgments = [{ id: 'old', attemptId: 'attempt', source: 'selfReport', outcome: 'correct', assessorVersion: 'learner-v1', supersedes: null }, { id: 'new', attemptId: 'attempt', source: 'selfReport', outcome: 'disputed', assessorVersion: 'learner-v1', supersedes: 'old', uncertainty: { kind: 'notEstimated' } }];
  const { port } = fixture(p); render(<ProgressPanel unitId="unit" port={port} />); await show();
  fireEvent.click(screen.getByText(/^Inspect attempt:/));
  expect(screen.getByText(/Superseded; retained as history/)).toBeInTheDocument();
  expect(screen.getByText(/Learner self-report: disputed/)).toBeInTheDocument();
});

it('hides withheld assessment judgments even if an adapter supplies them', async () => {
  const p = snapshot(); p.evidence[0].feedbackAvailable = false;
  p.evidence[0].judgments = [{ id: 'secret', attemptId: 'attempt', source: 'deterministic', outcome: 'correct', assessorVersion: 'secret-assessor', supersedes: null }];
  const { port } = fixture(p); render(<ProgressPanel unitId="unit" port={port} />); await show();
  fireEvent.click(screen.getByText(/^Inspect attempt:/));
  expect(screen.getByText(/Feedback withheld until the assessment is complete/)).toBeInTheDocument();
  expect(screen.queryByText(/secret-assessor/)).not.toBeInTheDocument();
});

it('announces page positions and returns focus to history on paging', async () => {
  const p = snapshot(); p.totalAttempts = 35; p.evidence = Array.from({ length: 32 }, (_, i) => evidence(`attempt-${i}`)); p.nextOffset = 32;
  const next = { ...p, offset: 32, nextOffset: null, evidence: [evidence('last')] };
  const { port, read } = fixture(p); read.mockResolvedValueOnce(p).mockResolvedValueOnce(next);
  render(<ProgressPanel unitId="unit" port={port} />); await show();
  fireEvent.click(screen.getByRole('button', { name: 'Next evidence page' }));
  await waitFor(() => expect(screen.getByRole('heading', { name: 'Evidence history' })).toHaveFocus());
  expect(read).toHaveBeenLastCalledWith('unit', 32);
  expect(screen.getByText(/Showing 33–33 of 35/)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Next evidence page' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Previous evidence page' })).toBeEnabled();
});

it('retains the last successful snapshot after a read failure and permits retry', async () => {
  const { port, read } = fixture(); render(<ProgressPanel unitId="unit" port={port} />); await show();
  read.mockRejectedValueOnce({ code: 'BudgetExceeded' }); fireEvent.click(screen.getByRole('button', { name: 'Refresh progress' }));
  await screen.findByRole('alert'); expect(screen.getByText(/^Inspect attempt:/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Refresh progress' }));
  await waitFor(() => expect(screen.queryByRole('alert')).not.toBeInTheDocument());
  expect(read).toHaveBeenCalledTimes(3);
});

it('rejects a response for a different lesson', async () => {
  const { port } = fixture({ ...snapshot(), unitId: 'foreign' }); render(<ProgressPanel unitId="unit" port={port} />);
  fireEvent.click(screen.getByRole('button', { name: 'Show lesson progress' })); await screen.findByRole('alert');
  expect(screen.queryByText(/^Inspect attempt:/)).not.toBeInTheDocument();
});

it.each(['lesson', 'course'])('ignores an in-flight response after switching %s', async kind => {
  let resolve: (value: LessonProgress) => void = () => { throw new Error('Missing pending read'); };
  const port: ProgressPort = { read: () => new Promise(value => { resolve = value; }) };
  const { rerender } = render(<ProgressPanel unitId="unit" port={port} />);
  fireEvent.click(screen.getByRole('button', { name: 'Show lesson progress' }));
  rerender(<ProgressPanel unitId={kind === 'lesson' ? 'other' : 'unit'} port={kind === 'lesson' ? port : fixture().port} />);
  await act(async () => { resolve(snapshot()); });
  expect(screen.queryByText(/^Inspect attempt:/)).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Show lesson progress' })).toHaveAttribute('aria-expanded', 'false');
});

it('distinguishes an empty history from achieved coverage', async () => {
  const p = { ...snapshot(), totalAttempts: 0, sessionCount: 0, evidence: [] };
  const { port } = fixture(p); render(<ProgressPanel unitId="unit" port={port} />); await show();
  expect(screen.getByText(/No study attempts recorded/)).toBeInTheDocument();
});
