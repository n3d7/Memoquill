import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { createNativeLessonPort } from '../lessons/nativeLessonPort';
import { StudyPanel } from './StudyPanel';
import type { StudyPort, StudySession, StudyView } from './types';

function session(mode: 'practice' | 'assessment' = 'practice'): StudySession {
  return { contractVersion: 1, id: 'session', unitId: 'unit', compositionRevisionId: 'composition', mode, revision: 2, cursor: 0,
    items: [{ bindingId: 'question', activity: { id: 'activity-revision', activityId: 'activity', unitId: 'unit', definition: { family: 'recall', version: 1, prompt: 'Say the word', answer: 'Answer', hints: ['First letter'], evaluation: 'exact' } } }],
    draft: { response: { kind: 'text', value: 'My draft' }, help: { hintsRevealed: 0, solutionRevealed: false, external: 'unknown' } } };
}
function fixture(s: StudySession) {
  const view: StudyView = { session: s, attempts: [], judgments: [], changedContent: false };
  const port: StudyPort = { sessions: vi.fn(async () => [s]), read: vi.fn(async () => view), apply: vi.fn(async () => s), recover: vi.fn(async () => undefined) };
  const lessons = createNativeLessonPort('course');
  return { port, view, lessons };
}

it('resumes an acknowledged draft and preserves the operation UUID on uncertain retry', async () => {
  const { port, lessons } = fixture(session());
  const apply = vi.mocked(port.apply); apply.mockRejectedValueOnce({ code: 'OutcomeUnconfirmed' });
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 0\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  expect(await screen.findByDisplayValue('My draft')).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Your response'), { target: { value: 'Changed response' } });
  expect(screen.getByRole('button', { name: 'Submit saved response' })).toBeDisabled();
  fireEvent.click(screen.getByRole('button', { name: 'Save response draft' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Reconnect and retry pending study action' }));
  await waitFor(() => expect(apply).toHaveBeenCalledTimes(2));
  expect(apply.mock.calls[1][0]).toEqual(apply.mock.calls[0][0]);
  expect(apply.mock.calls[0][0].action).toEqual({ kind: 'saveResponse', response: { kind: 'text', value: 'Changed response' }, externalAssistance: 'unknown' });
  expect(port.recover).toHaveBeenCalledOnce();
});

it('keeps assessment hints and references withheld while a session is incomplete', async () => {
  const { port, lessons } = fixture(session('assessment'));
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /assessment: 0\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  await screen.findByDisplayValue('My draft');
  expect(screen.queryByRole('button', { name: 'Reveal next hint' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Reveal reference' })).not.toBeInTheDocument();
  expect(screen.queryByText('Answer')).not.toBeInTheDocument();
});

it('appends a disputed self-assessment correction against the latest judgment and preserves history', async () => {
  const s = session(); s.cursor = 1; s.draft.response = null;
  const { port, view, lessons } = fixture(s);
  view.attempts = [{ id: 'attempt', sessionId: s.id, itemIndex: 0, activityRevisionId: 'activity-revision', response: { kind: 'text', value: 'My response' }, help: s.draft.help, selfRating: 4 }];
  view.judgments = [
    { id: 'deterministic', attemptId: 'attempt', source: 'deterministic', outcome: 'incorrect', assessorVersion: 'exact-v1', supersedes: null },
    { id: 'old', attemptId: 'attempt', source: 'selfReport', outcome: 'correct', assessorVersion: 'learner-v1', supersedes: null },
    { id: 'latest', attemptId: 'attempt', source: 'selfReport', outcome: 'unknown', assessorVersion: 'learner-v1', supersedes: 'old' },
  ];
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 1\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  const correction = await screen.findByLabelText('Correct self-assessment for response 1');
  await waitFor(() => expect(correction).toBeEnabled());
  fireEvent.change(correction, { target: { value: 'disputed' } });
  fireEvent.click(screen.getByRole('button', { name: 'Save self-assessment correction for response 1' }));
  await waitFor(() => expect(port.apply).toHaveBeenCalledOnce());
  expect(vi.mocked(port.apply).mock.calls[0][0].action).toEqual({ kind: 'correctSelfJudgment', attemptId: 'attempt', supersedes: 'latest', outcome: 'disputed' });
  expect(screen.getByText(/Self-assessment: correct/)).toBeInTheDocument();
  expect(screen.getByText(/time unknown/)).toBeInTheDocument();
});

it('renders bounded cloze as one literal blank without exposing the expected answer', async () => {
  const s = session();
  s.items[0].activity.definition = { family: 'recall', version: 2, responseFormat: 'boundedCloze', prompt: 'The word is {{answer}}.', answer: 'Secret answer', hints: [], evaluation: 'exact' };
  const { port, lessons } = fixture(s);
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 0\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  expect(await screen.findByText('The word is [blank].')).toBeInTheDocument();
  expect(screen.queryByText('Secret answer')).not.toBeInTheDocument();
});

it.each(['workedExample', 'completion'] as const)('shows pinned %s material as assisted practice without executing it', async phase => {
  const s = session();
  s.items[0].activity.definition = { family: 'constructed', version: 2, prompt: 'Explain or supply the missing step', rubric: 'Justify the reasoning', hints: [], evaluation: 'self', instruction: { phase, material: '<script>untrusted()</script> A partial solution' } };
  s.draft.help.solutionRevealed = true;
  const { port, lessons } = fixture(s);
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 0\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  expect(await screen.findByText('<script>untrusted()</script> A partial solution')).toBeInTheDocument();
  expect(screen.getByText(/recorded as assisted practice/)).toBeInTheDocument();
  expect(document.querySelector('script')).toBeNull();
  expect(screen.getByRole('button', { name: 'Reveal reference' })).toBeDisabled();
});

it('labels a flawed example and shows the pinned correction only after explicit reveal', async () => {
  const s = session();
  s.items[0].activity.definition = { family: 'constructed', version: 3, prompt: 'Repair the error', rubric: 'Explain the repair', hints: [], evaluation: 'self', instruction: { phase: 'errorRepair', material: '<script>untrusted()</script> 2 + 2 = 5', label: 'Intentional arithmetic error', expectedCorrection: 'Unique expected repair', review: { reviewer: 'Fixture author', reference: 'Addition reference' } } };
  const { port, lessons, view } = fixture(s);
  vi.mocked(port.apply).mockImplementation(async request => { if (request.action.kind === 'revealSolution') view.session.draft.help.solutionRevealed = true; return view.session; });
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 0\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  expect(await screen.findByText('Intentional arithmetic error')).toBeInTheDocument();
  expect(screen.getByText(/Review recorded by Fixture author/)).toBeInTheDocument();
  expect(screen.getByText(/do not establish independent factual verification/)).toBeInTheDocument();
  expect(screen.queryByText(/Unique expected repair/)).not.toBeInTheDocument();
  expect(document.querySelector('script')).toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Reveal reference' }));
  expect(await screen.findByText(/Unique expected repair/)).toBeInTheDocument();
  expect(port.apply).toHaveBeenCalledWith(expect.objectContaining({ action: { kind: 'revealSolution' } }));
});

it('keeps explicit FSRS recall ratings separate from general self-rating and reports unsaved navigation state', async () => {
  const s = session(); s.phase = 'independent'; s.draft.help.external = 'none';
  const { port, lessons } = fixture(s); const safe = vi.fn();
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} onSafeToLeave={safe} />);
  await screen.findByRole('option', { name: /practice: 0\/1/ });
  fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  await screen.findByLabelText('Recall schedule rating');
  fireEvent.change(screen.getByLabelText('Self-rating'), { target: { value: '1' } });
  fireEvent.change(screen.getByLabelText('Recall schedule rating'), { target: { value: '3' } });
  fireEvent.click(screen.getByRole('button', { name: 'Submit saved response' }));
  await waitFor(() => expect(port.apply).toHaveBeenCalled());
  expect(vi.mocked(port.apply).mock.calls[0][0].action).toEqual({ kind: 'submit', selfRating: 1, recallRating: 3, selfOutcome: 'unknown' });
  await waitFor(() => expect(safe).toHaveBeenLastCalledWith(true));
  fireEvent.change(screen.getByLabelText('Your response'), { target: { value: 'Unsaved learner text' } });
  await waitFor(() => expect(safe).toHaveBeenLastCalledWith(false));
});

it('reloads a conflicting session without losing the local response and saves against the refreshed revision', async () => {
  const s = session(); const { port, view, lessons } = fixture(s);
  const updated: StudyView = { ...view, session: { ...s, revision: 3, draft: { ...s.draft, response: { kind: 'text', value: 'Saved by another writer' } } } };
  const saved: StudyView = { ...updated, session: { ...updated.session, revision: 4, draft: { ...updated.session.draft, response: { kind: 'text', value: 'My retained response' } } } };
  vi.mocked(port.read).mockResolvedValueOnce(view).mockResolvedValueOnce(updated).mockResolvedValueOnce(saved);
  vi.mocked(port.apply).mockRejectedValueOnce({ code: 'revision_conflict' }).mockResolvedValueOnce(saved.session);
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 0\/1/ }); fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  await screen.findByDisplayValue('My draft'); fireEvent.change(screen.getByLabelText('Your response'), { target: { value: 'My retained response' } }); fireEvent.click(screen.getByRole('button', { name: 'Save response draft' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Reload saved session and retain my local response' }));
  expect(await screen.findByDisplayValue('Saved by another writer')).toBeInTheDocument(); expect(screen.getByText('My retained response')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Restore retained response for this activity' })); fireEvent.click(screen.getByRole('button', { name: 'Save response draft' }));
  await waitFor(() => expect(port.apply).toHaveBeenCalledTimes(2)); expect(vi.mocked(port.apply).mock.calls[1][0].expectedRevision).toBe(3);
  expect(vi.mocked(port.apply).mock.calls[1][0].action).toMatchObject({ kind: 'saveResponse', response: { kind: 'text', value: 'My retained response' } });
});

it('starts a lesson assessment with independent pins and explicitly excludes instructional scaffolds', async () => {
  const s = session(); const { port, view, lessons } = fixture(s);
  const { composition } = await import('../lessons/testFixtures'); const c = composition(); c.unitId = 'unit';
  c.bindings = [{ bindingId: 'worked', targetKind: 'activity', targetId: 'example', revisionId: 'example-revision' }, { bindingId: 'independent', targetKind: 'activity', targetId: 'activity', revisionId: 'activity-revision' }];
  vi.spyOn(lessons, 'read').mockResolvedValue({ id: 'composition', unitId: 'unit', composition: c, parentRevisionId: null, registryContractVersion: 1, hash: 'hash' });
  vi.spyOn(lessons, 'readActivity').mockResolvedValueOnce({ id: 'example-revision', activityId: 'example', unitId: 'unit', definition: { family: 'constructed', version: 2, prompt: 'Explain', rubric: 'Compare', hints: [], evaluation: 'self', instruction: { phase: 'workedExample', material: 'Sound reasoning' } } }).mockResolvedValueOnce(s.items[0].activity);
  vi.mocked(port.read).mockImplementation(async id => ({ ...view, session: { ...s, id, mode: 'assessment' } }));
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  fireEvent.click(screen.getByRole('button', { name: 'Start assessment' }));
  await waitFor(() => expect(port.apply).toHaveBeenCalledOnce());
  expect(vi.mocked(port.apply).mock.calls[0][0].action).toEqual({ kind: 'start', unitId: 'unit', compositionRevisionId: 'composition', mode: 'assessment', phase: 'independent', bindingIds: ['independent'] });
  expect(await screen.findByText(/Assessment excludes 1 instructional/)).toBeInTheDocument();
});

it.each([Number.MAX_SAFE_INTEGER, 8_640_000_000_000_001])('retains a valid persisted timestamp outside Date display range without crashing: %s', async submittedAtUnixMs => {
  const s = session(); s.cursor = 1; s.draft.response = null;
  const { port, view, lessons } = fixture(s);
  view.attempts = [{ id: 'attempt', sessionId: s.id, itemIndex: 0, activityRevisionId: s.items[0].activity.id, response: { kind: 'text', value: 'Answer' }, help: s.draft.help, selfRating: null, submittedAtUnixMs }];
  render(<StudyPanel unitId="unit" lessons={lessons} port={port} />);
  await screen.findByRole('option', { name: /practice: 1\/1/ }); fireEvent.change(screen.getByLabelText('Saved study sessions'), { target: { value: 'session' } });
  expect(await screen.findByText(new RegExp(`${submittedAtUnixMs} UTC Unix ms`))).toHaveTextContent('outside supported date display range');
  expect(port.apply).not.toHaveBeenCalled();
});
