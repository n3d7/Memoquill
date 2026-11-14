import { activityPrompt } from '../lessons/activityPrompt';
import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../application/errors';
import type { LessonPort } from '../lessons/types';
import type { AssessmentOutcome, DeclaredAssistance, StudyAction, StudyMutation, StudyPort, StudyResponse, StudySession, StudyView, StudyPhase } from './types';

export function StudyPanel({ unitId, lessons, port, initialSessionId, onSafeToLeave }: { unitId: string; lessons: LessonPort; port: StudyPort; initialSessionId?: string; onSafeToLeave?: (safe: boolean) => void }) {
  const [sessions, setSessions] = useState<StudySession[]>([]); const [view, setView] = useState<StudyView | null>(null);
  const [nextOffset, setNextOffset] = useState<number | null>(null);
  const [response, setResponse] = useState<StudyResponse | null>(null); const [external, setExternal] = useState<DeclaredAssistance>('unknown');
  const [dirty, setDirty] = useState(false); const [rating, setRating] = useState(''); const [outcome, setOutcome] = useState<AssessmentOutcome>('unknown');
  const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const [pending, setPending] = useState<StudyMutation | null>(null);
  const [notice, setNotice] = useState('');
  const [needsReload, setNeedsReload] = useState(false);
  const [retained, setRetained] = useState<{ index: number; activity: string; response: StudyResponse; external: DeclaredAssistance; rating: string; recallRating: string; outcome: AssessmentOutcome } | null>(null);
  const [recallRating, setRecallRating] = useState('');
  const [phase, setPhase] = useState<StudyPhase>('independent');
  const lock = useRef(false); const mounted = useRef(true);
  useEffect(() => { mounted.current = true; let current = true; void port.sessions(unitId).then(value => { if (current) { setSessions(value); setNextOffset(value.length === 64 ? 64 : null); } }, reason => { if (current) setError(failureMessage(reason)); }); return () => { current = false; mounted.current = false; }; }, [port, unitId]);
  useEffect(() => { if (initialSessionId) void open(initialSessionId); }, [initialSessionId]);
  useEffect(() => { onSafeToLeave?.((view !== null || error !== '') && !busy && !dirty && pending === null && retained === null && rating === '' && recallRating === '' && outcome === 'unknown'); }, [view, error, busy, dirty, pending, retained, rating, recallRating, outcome, onSafeToLeave]);
  async function older() {
    if (lock.current || nextOffset === null) return; lock.current = true; setBusy(true); setError('');
    try { const page = await port.sessions(unitId, nextOffset); if (mounted.current) { setSessions(old => [...old, ...page.filter(s => !old.some(prior => prior.id === s.id))]); setNextOffset(page.length === 64 ? nextOffset + 64 : null); } }
    catch (reason) { if (mounted.current) setError(failureMessage(reason)); }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  function accept(value: StudyView) {
    if (value.session.unitId !== unitId) throw new Error('Study scope mismatch');
    setNeedsReload(false); setView(value); setResponse(value.session.draft.response); setExternal(value.session.draft.help.external); setDirty(false); setRating(''); setRecallRating(''); setOutcome('unknown');
    setSessions(old => [value.session, ...old.filter(session => session.id !== value.session.id)]);
  }
  async function open(id: string) {
    if (lock.current || pending || dirty) return; lock.current = true; setBusy(true); setError(''); setNotice('');
    try { const value = await port.read(id); if (mounted.current) accept(value); } catch (reason) { if (mounted.current) setError(failureMessage(reason)); }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  async function run(request: StudyMutation, recover = false) {
    if (lock.current) return; lock.current = true; setBusy(true); setPending(request); setError('');
    try {
      if (recover) await port.recover();
      await port.apply(request);
      const value = await port.read(request.sessionId);
      if (mounted.current) { accept(value); setPending(null); }
    } catch (reason) {
      if (mounted.current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) { setPending(null); setNeedsReload(typeof reason === 'object' && reason !== null && 'code' in reason && reason.code === 'revision_conflict'); } }
    } finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  async function reloadConflict() {
    if (!view || lock.current || pending) return;
    const backup = response ? { index: view.session.cursor, activity: view.session.items[view.session.cursor]?.activity.id ?? '', response, external, rating, recallRating, outcome } : null;
    lock.current = true; setBusy(true); setError('');
    try { const value = await port.read(view.session.id); if (mounted.current) { accept(value); setRetained(backup); } }
    catch (reason) { if (mounted.current) setError(failureMessage(reason)); }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  function action(value: StudyAction) {
    if (!view || busy || pending) return;
    void run({ operationId: crypto.randomUUID(), sessionId: view.session.id, expectedRevision: view.session.revision, action: value });
  }
  async function start(mode: 'practice' | 'assessment') {
    if (lock.current || pending || dirty) return; lock.current = true; setBusy(true); setError('');
    let request: StudyMutation | null = null;
    try {
      const saved = await lessons.read(unitId);
      if (!saved || saved.unitId !== unitId) throw new Error('Publish this lesson with at least one activity before studying.');
      let bindingIds: string[] | undefined;
      setNotice('');
      if (mode === 'assessment') {
        const bindings = saved.composition.bindings.filter(binding => binding.targetKind === 'activity');
        const activities = await Promise.all(bindings.map(binding => lessons.readActivity(unitId, binding.revisionId)));
        bindingIds = bindings.filter((binding, index) => {
          const activity = activities[index];
          if (!activity || activity.unitId !== unitId || activity.id !== binding.revisionId) throw new Error('A pinned activity is unavailable. Reload this lesson before starting an assessment.');
          const definition = activity.definition;
          return definition.version === 1 && ['recall', 'singleChoice', 'constructed', 'procedure'].includes(definition.family) || definition.family === 'recall' && definition.version === 2;
        }).map(binding => binding.bindingId);
        if (bindingIds.length === 0) throw new Error('Publish at least one independent activity before starting an assessment. Worked examples and completion scaffolds are practice only.');
        if (mounted.current && bindingIds.length < bindings.length) setNotice(`Assessment excludes ${bindings.length - bindingIds.length} instructional or unsupported activities and keeps the selected independent questions fixed.`);
      }
      request = { operationId: crypto.randomUUID(), sessionId: crypto.randomUUID(), expectedRevision: 0, action: { kind: 'start', unitId, compositionRevisionId: saved.id, mode, phase, ...(bindingIds ? { bindingIds } : {}) } };
    } catch (reason) { if (mounted.current) setError(failureMessage(reason)); }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
    if (request && mounted.current) await run(request);
  }
  const session = view?.session; const item = session?.items[session.cursor]; const definition = item?.activity.definition;
  const canSchedule = session?.mode === 'practice' && (session.phase === 'independent' || session.phase === 'delayedCheck') && definition?.family === 'recall' && session.draft.help.hintsRevealed === 0 && !session.draft.help.solutionRevealed && external === 'none';
  const blocked = busy || pending !== null || retained !== null; const feedback = view?.feedbackAvailable ?? (session?.mode === 'practice' || session?.cursor === session?.items.length);
  function edit(value: StudyResponse) { setResponse(value); setDirty(true); }
  return <section aria-label="Study lesson"><h3>Study this lesson</h3>
    <p>Practice keeps hints and self-assessment separate from your response. Assessment withholds hints and feedback until the session ends.</p>
    {!initialSessionId && <><label>Study context<select value={phase} disabled={blocked || dirty} onChange={e => setPhase(e.target.value as StudyPhase)}><option value="independent">Independent attempt</option><option value="guided">Guided practice</option><option value="delayedCheck">Delayed check</option><option value="changedContext">Changed-context check</option></select></label>
    <button disabled={blocked || dirty} onClick={() => void start('practice')}>Start practice</button>{' '}
    <button disabled={blocked || dirty} onClick={() => void start('assessment')}>Start assessment</button></>}
    {!initialSessionId && sessions.length > 0 && <label>Saved study sessions<select value={session?.id ?? ''} disabled={blocked || dirty} onChange={e => { if (e.target.value) void open(e.target.value); }}><option value="">Choose session to resume</option>{sessions.map(s => <option key={s.id} value={s.id}>{s.mode}: {s.cursor}/{s.items.length} completed — {s.id.slice(0, 8)}</option>)}</select></label>}
    {!initialSessionId && nextOffset !== null && <button disabled={blocked} onClick={() => void older()}>Load older study sessions</button>}
    {notice && <p role="status">{notice}</p>}
    {busy && <p role="status">Saving or loading study state…</p>}{error && <p role="alert">{error}</p>}
    {initialSessionId && !view && !busy && error && !pending && <button onClick={() => void open(initialSessionId)}>Reload saved study session</button>}
    {pending && !busy && <button onClick={() => void run(pending, true)}>Reconnect and retry pending study action</button>}
    {needsReload && view && !busy && !pending && <button onClick={() => void reloadConflict()}>Reload saved session and retain my local response</button>}
    {retained && <aside aria-label="Retained local response"><p>Your local response is retained in this tab. Review the saved version before restoring it.</p><pre>{retained.response.kind === 'text' ? retained.response.value : retained.response.kind === 'choice' ? retained.response.optionId : retained.response.observations}</pre>
      <button disabled={busy || pending !== null || session?.cursor !== retained.index || item?.activity.id !== retained.activity} onClick={() => { setResponse(retained.response); setExternal(retained.external); setRating(retained.rating); setRecallRating(retained.recallRating); setOutcome(retained.outcome); setDirty(true); setRetained(null); }}>Restore retained response for this activity</button>
      <button disabled={busy || pending !== null} onClick={() => setRetained(null)}>Dismiss retained local response</button>
    </aside>}
    {view?.changedContent && <p role="status">This lesson has changed. This session keeps its original questions and assessment references.</p>}
    {session && <p>Completed {session.cursor} of {session.items.length}. {dirty ? 'Response has unsaved changes.' : 'Acknowledged responses are saved.'}</p>}
    {definition && session && <fieldset disabled={blocked}><legend>Current activity</legend><p>{activityPrompt(definition)}</p>
      {definition.family === 'constructed' && definition.version === 2 && session.mode === 'practice' && <aside aria-label="Instructional support"><h4>{definition.instruction.phase === 'workedExample' ? 'Worked example — explain the reasoning' : 'Completion — supply the missing steps'}</h4><pre>{definition.instruction.material}</pre><p>This response uses a shown solution or scaffold. It is recorded as assisted practice.</p></aside>}
      {(definition.family === 'recall' || definition.family === 'constructed') && <label>Your response<textarea maxLength={16384} value={response?.kind === 'text' ? response.value : ''} onChange={e => edit({ kind: 'text', value: e.target.value })} /></label>}
      {definition.family === 'singleChoice' && <fieldset><legend>Choose one answer</legend>{definition.options.map(option => <label key={option.id}><input type="radio" name={`study-${session.id}-${session.cursor}`} value={option.id} checked={response?.kind === 'choice' && response.optionId === option.id} onChange={() => edit({ kind: 'choice', optionId: option.id })} />{option.text}</label>)}</fieldset>}
      {definition.family === 'procedure' && <><p>Record what you did and observed. Completion is self-report.</p>{definition.steps.map(step => <label key={step.id}><input type="checkbox" checked={response?.kind === 'procedure' && response.completedStepIds.includes(step.id)} onChange={e => {
        const prior = response?.kind === 'procedure' ? response : { kind: 'procedure' as const, completedStepIds: [], observations: '' };
        edit({ ...prior, completedStepIds: e.target.checked ? [...prior.completedStepIds, step.id] : prior.completedStepIds.filter(id => id !== step.id) });
      }} />{step.text}</label>)}<label>Observations<textarea maxLength={16384} value={response?.kind === 'procedure' ? response.observations : ''} onChange={e => edit({ kind: 'procedure', completedStepIds: response?.kind === 'procedure' ? response.completedStepIds : [], observations: e.target.value })} /></label></>}
      <label>External help<select value={external} onChange={e => { setExternal(e.target.value as DeclaredAssistance); setDirty(true); }}><option value="unknown">Unknown / not reported</option><option value="none">I used no external help</option><option value="present">I used external help</option></select></label>
      <button disabled={!response || !dirty} onClick={() => { if (response) action({ kind: 'saveResponse', response, externalAssistance: external }); }}>Save response draft</button>
      <p>Save changes before revealing help or submitting.</p>
      {session.mode === 'practice' && <><button disabled={dirty || session.draft.help.hintsRevealed >= definition.hints.length} onClick={() => action({ kind: 'revealHint' })}>Reveal next hint</button><button disabled={dirty || session.draft.help.solutionRevealed} onClick={() => action({ kind: 'revealSolution' })}>Reveal reference</button>
        {definition.hints.slice(0, session.draft.help.hintsRevealed).map((hint, i) => <p key={i}>{hint}</p>)}
        {session.draft.help.solutionRevealed && <p>{reference(definition)}</p>}</>}
      <label>Self-rating<select value={rating} onChange={e => setRating(e.target.value)}><option value="">Not rated</option><option value="1">1 — difficult</option><option value="2">2</option><option value="3">3</option><option value="4">4 — easy</option></select></label>
      {canSchedule && <label>Recall schedule rating<select value={recallRating} onChange={e => setRecallRating(e.target.value)}><option value="">Save without scheduling</option><option value="1">Again — I could not recall</option><option value="2">Hard — I recalled with difficulty</option><option value="3">Good — I recalled</option><option value="4">Easy — I recalled easily</option></select></label>}
      <p>Only an explicitly rated, unassisted repeatable recall updates FSRS. Your general self-rating and assessment stay separate.</p>
      <label>Your assessment<select value={outcome} onChange={e => setOutcome(e.target.value as AssessmentOutcome)}><option value="unknown">Unknown</option><option value="disputed">Disputed</option><option value="correct">Meets the reference / rubric</option><option value="incorrect">Does not meet the reference / rubric</option></select></label>
      <button disabled={dirty || !session.draft.response} onClick={() => action({ kind: 'submit', selfRating: rating ? Number(rating) : null, selfOutcome: outcome, ...(canSchedule && recallRating ? { recallRating: Number(recallRating) } : {}) })}>Submit saved response</button>
    </fieldset>}
    {session?.assessmentId && !feedback && <p role="status">Feedback is withheld until every part of this assessment is complete.</p>}
    {session && !item && <p role="status">Session completed. Responses and assessment history are retained.</p>}
    {view && feedback && view.attempts.length > 0 && <section aria-label="Study evidence"><h4>Recorded responses and feedback</h4><p>Choice recognition, help, self-report and deterministic checks describe different evidence. These results are not a universal mastery score.</p>{view.attempts.map(attempt => {
      const revision = session?.items[attempt.itemIndex].activity.definition;
      const judgments = view.judgments.filter(j => j.attemptId === attempt.id);
      const latestSelf = judgments.find(j => j.source === 'selfReport' && !judgments.some(next => next.supersedes === j.id));
      return <article key={attempt.id}><h5>Response {attempt.itemIndex + 1}</h5><pre>{attempt.response.kind === 'text' ? attempt.response.value : attempt.response.kind === 'choice' ? attempt.response.optionId : attempt.response.observations}</pre>
        <p>Hints: {attempt.help.hintsRevealed}. Reference revealed: {attempt.help.solutionRevealed ? 'yes' : 'no'}. External help: {attempt.help.external}. Self-rating: {attempt.selfRating ?? 'not rated'}.</p>
        <p>Instructional phase: {revision?.family === 'constructed' && revision.version === 2 ? revision.instruction.phase : 'independent response'}. Context: {session?.phase ?? 'unspecified'}. Submitted: {observedTime(attempt.submittedAtUnixMs)}. Timestamps do not measure active study or prove retention.</p>
        {revision && <p>Reference: {reference(revision)}</p>}{judgments.map(j => <p key={j.id}>{j.source === 'selfReport' ? 'Self-assessment' : j.source === 'model' ? 'Model opinion' : 'Deterministic check'}: {j.outcome}{j.supersedes ? ' (correction; original retained)' : ''}. Assessor: {j.assessorVersion}. Uncertainty: {j.uncertainty?.kind ?? 'notEstimated'}.</p>)}
        {latestSelf && <SelfCorrection key={latestSelf.id} index={attempt.itemIndex} disabled={blocked || dirty} onCorrect={value => action({ kind: 'correctSelfJudgment', attemptId: attempt.id, supersedes: latestSelf.id, outcome: value })} />}
      </article>;
    })}</section>}
  </section>;
}

function reference(definition: import('../lessons/types').ActivityDefinition): string {
  switch (definition.family) {
    case 'recall': return definition.answer;
    case 'singleChoice': return definition.options.find(o => o.id === definition.correctOptionId)?.text ?? 'Unknown reference';
    case 'constructed': return definition.rubric;
    case 'procedure': return definition.reflection;
  }
}

function observedTime(value: number | null | undefined): string {
  if (value == null) return 'time unknown';
  const date = new Date(value);
  return Number.isFinite(date.getTime()) ? date.toISOString() : `${value} UTC Unix ms (outside supported date display range)`;
}

function SelfCorrection({ index, disabled, onCorrect }: { index: number; disabled: boolean; onCorrect: (outcome: AssessmentOutcome) => void }) {
  const [value, setValue] = useState<AssessmentOutcome>('unknown');
  return <fieldset disabled={disabled}><legend>Append a correction; keep the original assessment</legend><label>Correct self-assessment for response {index + 1}<select value={value} onChange={event => setValue(event.target.value as AssessmentOutcome)}><option value="unknown">Unknown</option><option value="disputed">Disputed</option><option value="correct">Meets the reference / rubric</option><option value="incorrect">Does not meet the reference / rubric</option></select></label><button onClick={() => onCorrect(value)}>Save self-assessment correction for response {index + 1}</button></fieldset>;
}
