import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { createNativeLessonPort } from '../lessons/nativeLessonPort';
import { ScopedAssessmentPanel } from './ScopedAssessmentPanel';
import type { AssessmentPort, AssessmentView, FrozenAssessment } from './assessmentTypes';
import type { StudyPort } from './types';

function fixture() {
  const run: FrozenAssessment = { contractVersion: 1, id: 'run', scope: { kind: 'subject', id: 'subject' }, title: 'Mathematics', feedbackPolicy: 'afterAllPartsComplete', createdAtUnixMs: null, parts: [{ sessionId: 'pinned-session', unitId: 'unit', unitTitle: 'Algebra', objective: 'Explain the reasoning', compositionRevisionId: 'pinned-composition', items: [{ bindingId: 'binding', activityRevisionId: 'pinned-activity', family: 'constructed' }] }], omissions: [] };
  const view: AssessmentView = { assessment: run, parts: [{ sessionId: 'pinned-session', started: false, completed: 0, total: 1, changedContent: true }], feedbackAvailable: false, coverage: [{ outcome: 'constructedResponse', total: 1, attempted: 0, unattempted: 1, declaredUnaided: 0, externalUnknown: 0, externalPresent: 0, feedback: null }] };
  const port: AssessmentPort = { create: vi.fn(async () => run), read: vi.fn(async () => view), list: vi.fn(async () => [{ id: 'run', title: 'Mathematics', partCount: 1, itemCount: 1 }]), recover: vi.fn(async () => undefined) };
  const study: StudyPort = { apply: vi.fn(), read: vi.fn(), sessions: vi.fn(async () => []), recover: vi.fn() };
  const props = { port, study, lessons: createNativeLessonPort('course'), subjects: { subject: { title: 'Mathematics', archived: false } }, topics: {} };
  return { port, study, run, view, props };
}
it('retains the same assessment identity and operation on uncertain creation retry', async () => {
  const { port, run, view, props } = fixture();
  vi.mocked(port.create).mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementationOnce(async request => { run.id = request.assessmentId; return run; });
  vi.mocked(port.read).mockImplementation(async () => view);
  render(<ScopedAssessmentPanel {...props} />);
  fireEvent.change(screen.getByLabelText('Subject or topic'), { target: { value: 'subject' } });
  fireEvent.click(screen.getByRole('button', { name: 'Create assessment' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Reconnect and retry assessment creation' }));
  await waitFor(() => expect(port.create).toHaveBeenCalledTimes(2));
  expect(vi.mocked(port.create).mock.calls[1][0]).toEqual(vi.mocked(port.create).mock.calls[0][0]);
  expect(port.recover).toHaveBeenCalledOnce();
});
it('keeps grading hidden before global completion and presents changed-content and unknown coverage separately', async () => {
  const { view, props } = fixture();
  view.coverage[0].feedback = { deterministicCorrect: 99, deterministicIncorrect: 0, deterministicUnknown: 0, selfCorrect: 99, selfIncorrect: 0, selfUnknown: 0, selfDisputed: 0 };
  render(<ScopedAssessmentPanel {...props} />);
  await screen.findByRole('option', { name: /Mathematics: 1 lessons/ });
  fireEvent.change(screen.getByLabelText('Saved assessments'), { target: { value: 'run' } });
  expect(await screen.findByText(/Feedback is withheld until every assessment part/)).toBeInTheDocument();
  expect(screen.getByText(/original questions are retained/)).toBeInTheDocument();
  expect(screen.getByText(/1 unattempted/)).toBeInTheDocument();
  expect(screen.queryByText(/99 correct/)).not.toBeInTheDocument();
});
