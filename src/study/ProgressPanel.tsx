import { useEffect, useId, useRef, useState } from 'react';
import { failureMessage } from '../application/errors';
import type { ProgressPort, LessonProgress, ProgressEvidence } from './progressTypes';
import type { AssessmentJudgment } from './types';

const outcomes: Record<string, string> = { recall: 'Recall', recognition: 'Recognition / choice', constructedResponse: 'Constructed response / problem', practicalSelfReport: 'Practical self-report', unknown: 'Unknown outcome' };
const sources: Record<AssessmentJudgment['source'], string> = { deterministic: 'Deterministic check', selfReport: 'Learner self-report', model: 'Model judgment' };
function time(value: number | null | undefined) {
  if (value == null) return 'Not recorded';
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? 'Not recorded' : date.toISOString();
}
function Evidence({ evidence: e }: { evidence: ProgressEvidence }) {
  const superseded = new Set(e.judgments.flatMap(j => j.supersedes ? [j.supersedes] : []));
  const response = e.attempt.response;
  return <details><summary>Inspect attempt: {outcomes[e.outcome] ?? outcomes.unknown} — {time(e.attempt.submittedAtUnixMs)}</summary>
    <p>{e.mode === 'practice' ? 'Practice' : 'Assessment'}; phase: {e.phase === 'unspecified' ? 'Not recorded' : e.phase}.</p>
    <p>{e.currentActivity ? 'Current activity revision.' : 'Previous or removed activity revision.'} {e.changedComposition && 'Lesson composition has changed; this record retains its original context.'}</p>
    <p>Hints revealed: {e.attempt.help.hintsRevealed}. Solution revealed: {e.attempt.help.solutionRevealed ? 'Yes' : 'No'}. External assistance: {e.attempt.help.external === 'unknown' ? 'Unknown; not verified unaided' : e.attempt.help.external === 'none' ? 'Learner declared none; not independently verified' : 'Learner declared present'}.</p>
    <p>Response: {response.kind === 'text' ? response.value : response.kind === 'choice' ? `Selected option: ${response.optionId}` : `${response.completedStepIds.length} steps reported complete. ${response.observations}`}</p>
    <p>Self-rating: {e.attempt.selfRating ?? 'Not recorded'}. Recall rating: {e.attempt.recallRating ?? 'Not recorded'}. These ratings are not independent correctness.</p>
    <p>First saved: {time(e.attempt.firstSavedAtUnixMs)}. Submitted: {time(e.attempt.submittedAtUnixMs)}. These timestamps do not measure active learning time.</p>
    {!e.feedbackAvailable ? <p>Feedback withheld until the assessment is complete.</p> : e.judgments.length === 0 ? <p>Unassessed: no judgment recorded.</p> : <ul>{e.judgments.map(j => <li key={j.id}>{sources[j.source]}: {j.outcome}. {superseded.has(j.id) ? 'Superseded; retained as history.' : 'Current judgment.'} Uncertainty: {j.uncertainty?.kind === 'reported' ? `${j.uncertainty.confidenceBasisPoints / 100}% reported confidence; ${j.uncertainty.note}` : j.uncertainty?.kind === 'notApplicable' ? 'Not applicable' : 'Not estimated'}. Assessor: {j.assessorVersion}. Recorded: {time(j.recordedAtUnixMs)}.{j.provenance ? <span> Definition digest: {j.provenance.definitionSha256}; activity: {j.provenance.activityRevisionId}; composition: {j.provenance.compositionRevisionId}; assessor kind: {j.provenance.assessor.kind}.</span> : ' Provenance not recorded.'}</li>)}</ul>}
    <dl><dt>Attempt</dt><dd>{e.attempt.id}</dd><dt>Session</dt><dd>{e.attempt.sessionId}</dd><dt>Activity revision</dt><dd>{e.attempt.activityRevisionId}</dd><dt>Lesson composition revision</dt><dd>{e.compositionRevisionId}</dd></dl>
    {e.sources.length ? <ul aria-label="Pinned source references">{e.sources.map(s => <li key={s.bindingId}>Source reference: {s.targetId}; revision: {s.revisionId}; location: {s.locator ?? 'Not recorded'}.</li>)}</ul> : <p>No pinned source reference recorded.</p>}
    <p>Source references do not establish that material was read or extracted.</p>
  </details>;
}

/** A read-only view. It never starts, resumes or modifies a study session. */
export function ProgressPanel({ unitId, port }: { unitId: string; port: ProgressPort }) {
  const id = useId();
  const [view, setView] = useState<LessonProgress | null>(null);
  const [opened, setOpened] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const heading = useRef<HTMLHeadingElement>(null);
  const epoch = useRef(0);
  const pending = useRef(false);
  useEffect(() => { epoch.current += 1; pending.current = false; setView(null); setOpened(false); setBusy(false); setError(''); setNotice(''); return () => { epoch.current += 1; pending.current = false; }; }, [unitId, port]);
  async function read(offset: number, moveFocus = false) {
    if (pending.current) return;
    pending.current = true; const requestEpoch = epoch.current;
    setBusy(true); setError('');
    try {
      const next = await port.read(unitId, offset);
      if (epoch.current !== requestEpoch) return;
      if (next.unitId !== unitId || next.offset !== offset || next.contractVersion !== 1) throw new Error('Progress scope mismatch');
      setNotice(view && (view.totalAttempts !== next.totalAttempts || view.currentCompositionRevisionId !== next.currentCompositionRevisionId) ? 'Evidence changed since the previous read. This page shows the current snapshot.' : '');
      setView(next);
      if (moveFocus) heading.current?.focus();
    } catch (reason) {
      if (epoch.current === requestEpoch) setError(failureMessage(reason));
    } finally {
      if (epoch.current === requestEpoch) { pending.current = false; setBusy(false); }
    }
  }
  return <section aria-label="Lesson progress"><h3>Lesson progress</h3><button aria-expanded={opened} aria-controls={id} onClick={() => { setOpened(!opened); if (!opened) void read(0); }}>{opened ? 'Hide lesson progress' : 'Show lesson progress'}</button>
    <div id={id} hidden={!opened} aria-busy={busy}>
      <p>Evidence by outcome and activity conditions; counts are not universal mastery or verified competence.</p>
      <button disabled={busy} onClick={() => void read(0, true)}>Refresh progress</button>
      {error && <p role="alert">{error} The last successful snapshot, if present, remains below.</p>}
      {notice && <p role="status">{notice}</p>}
      <h4 ref={heading} tabIndex={-1}>Evidence history</h4>
      {view && <><p role="status">{view.totalAttempts} recorded attempts in {view.sessionCount} sessions. Showing {view.evidence.length ? view.offset + 1 : 0}–{view.offset + view.evidence.length} of {view.totalAttempts}.</p>
        {!view.totalAttempts && <p>No study attempts recorded. Coverage below describes current activity bindings, not achievement.</p>}
        {!view.currentCompositionRevisionId && <p>No current published lesson composition.</p>}
        <ul aria-label="Outcome coverage">{view.coverage.map(row => <li key={row.outcome}><strong>{outcomes[row.outcome] ?? outcomes.unknown}</strong>: {row.evidencedCurrentActivityBindings} of {row.currentActivityBindings} current activity bindings have a submitted attempt. {row.currentActivityBindings - row.evidencedCurrentActivityBindings} not attempted. {row.attempts} attempts in history; {row.previousActivityAttempts} for previous or removed activity revisions; {row.feedbackWithheldAttempts} with feedback withheld; {row.unassessedAttempts} unassessed.{row.judgments.length > 0 && <ul aria-label={`${outcomes[row.outcome] ?? outcomes.unknown} judgments`}>{row.judgments.map(j => <li key={`${j.source}-${j.outcome}`}>{sources[j.source]}: {j.outcome} — {j.count}</li>)}</ul>}</li>)}</ul>
        <ol start={view.offset + 1}>{view.evidence.map(e => <li key={e.attempt.id}><Evidence evidence={e} /></li>)}</ol>
        <nav aria-label="Evidence pages"><button disabled={busy || view.offset === 0} onClick={() => void read(Math.max(0, view.offset - 32), true)}>Previous evidence page</button><button disabled={busy || view.nextOffset === null} onClick={() => { if (view.nextOffset !== null) void read(view.nextOffset, true); }}>Next evidence page</button></nav>
      </>}
    </div>
  </section>;
}
