import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../application/errors';
import type { LessonPort } from '../lessons/types';
import type { AssessmentPort, AssessmentSummary, AssessmentView, CreateAssessment } from './assessmentTypes';
import type { StudyMutation, StudyPort } from './types';
import { StudyPanel } from './StudyPanel';

export function ScopedAssessmentPanel({ port, study, lessons, subjects, topics }: { port: AssessmentPort; study: StudyPort; lessons: LessonPort; subjects: Record<string, { title: string; archived: boolean }>; topics: Record<string, { title: string; archived: boolean; subjectId: string }> }) {
  const [kind, setKind] = useState<'subject' | 'topic'>('subject'); const [scope, setScope] = useState(''); const [saved, setSaved] = useState<AssessmentSummary[]>([]); const [offset, setOffset] = useState<number | null>(null);
  const [view, setView] = useState<AssessmentView | null>(null); const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  const [pending, setPending] = useState<CreateAssessment | null>(null); const [pendingPart, setPendingPart] = useState<StudyMutation | null>(null); const [active, setActive] = useState<number | null>(null); const [canLeave, setCanLeave] = useState(false);
  const lock = useRef(false); const mounted = useRef(true);
  useEffect(() => { mounted.current = true; let current = true; void port.list().then(values => { if (current) { setSaved(values); setOffset(values.length === 64 ? 64 : null); } }, reason => { if (current) setError(failureMessage(reason)); }); return () => { current = false; mounted.current = false; }; }, [port]);
  async function refresh(id: string) { const value = await port.read(id); if (value.assessment.id !== id) throw new Error('Assessment scope mismatch'); if (mounted.current) setView(value); }
  async function read(id: string) {
    if (lock.current || pending || pendingPart || active !== null) return; lock.current = true; setBusy(true); setError('');
    try { await refresh(id); } catch (reason) { if (mounted.current) setError(failureMessage(reason)); } finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  async function create(request: CreateAssessment, recover = false) {
    if (lock.current) return; lock.current = true; setBusy(true); setPending(request); setError('');
    try { if (recover) await port.recover(); const run = await port.create(request); if (run.id !== request.assessmentId) throw new Error('Assessment identity mismatch'); await refresh(run.id); if (mounted.current) { setPending(null); setSaved(old => [{ id: run.id, title: run.title, partCount: run.parts.length, itemCount: run.parts.reduce((n,p) => n + p.items.length, 0) }, ...old.filter(v => v.id !== run.id)]); } }
    catch (reason) { if (mounted.current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) setPending(null); } }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  async function startPart(request: StudyMutation, index: number, recover = false) {
    if (lock.current) return; lock.current = true; setBusy(true); setPendingPart(request); setError('');
    try { if (recover) await study.recover(); await study.apply(request); if (view) await refresh(view.assessment.id); if (mounted.current) { setCanLeave(false); setActive(index); setPendingPart(null); } }
    catch (reason) { if (mounted.current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) { setPendingPart(null); if (view) await refresh(view.assessment.id).catch(() => {}); } } }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  async function older() {
    if (lock.current || offset === null) return; lock.current = true; setBusy(true);
    try { const values = await port.list(offset); if (mounted.current) { setSaved(old => [...old, ...values.filter(v => !old.some(p => p.id === v.id))]); setOffset(values.length === 64 ? offset + 64 : null); } } catch (reason) { if (mounted.current) setError(failureMessage(reason)); } finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  const blocked = busy || pending !== null || pendingPart !== null; const selected = active === null ? null : view?.assessment.parts[active];
  const scopes = kind === 'subject' ? subjects : topics;
  return <section aria-label="Topic and subject assessments"><h3>Topic and subject assessments</h3>
    <p>Assess active lessons placed in a subject or a topic and its subtopics. Unplaced lessons are outside this scope. Questions and feedback policy are fixed at creation; feedback waits until every part is complete.</p>
    <form onSubmit={e => { e.preventDefault(); void create({ operationId: crypto.randomUUID(), assessmentId: crypto.randomUUID(), scope: { kind, id: scope } }); }}><fieldset disabled={blocked || active !== null}><legend>Create an assessment</legend>
      <label>Assessment scope<select value={kind} onChange={e => { setKind(e.target.value as 'subject' | 'topic'); setScope(''); }}><option value="subject">Subject</option><option value="topic">Topic and subtopics</option></select></label>
      <label>Subject or topic<select value={scope} onChange={e => setScope(e.target.value)}><option value="">Choose scope</option>{Object.entries(scopes).filter(([,v]) => !v.archived).map(([id,v]) => <option key={id} value={id}>{v.title}</option>)}</select></label><button disabled={!scope}>Create assessment</button>
    </fieldset></form>
    <label>Saved assessments<select disabled={blocked || active !== null} value={view?.assessment.id ?? ''} onChange={e => { if (e.target.value) void read(e.target.value); }}><option value="">Choose assessment</option>{saved.map(v => <option key={v.id} value={v.id}>{v.title}: {v.partCount} lessons, {v.itemCount} questions — {v.id.slice(0,8)}</option>)}</select></label>
    {offset !== null && <button disabled={blocked || active !== null} onClick={() => void older()}>Load older assessments</button>}
    {busy && <p role="status">Saving or loading assessment state…</p>}{error && <p role="alert">{error}</p>}
    {pending && !busy && <button onClick={() => void create(pending, true)}>Reconnect and retry assessment creation</button>}
    {pendingPart?.action.kind === 'startScopedAssessment' && !busy && <button onClick={() => { if (pendingPart.action.kind === 'startScopedAssessment') void startPart(pendingPart, pendingPart.action.partIndex, true); }}>Reconnect and retry assessment part</button>}
    {view && <><h4>{view.assessment.title}</h4><p>{view.feedbackAvailable ? 'All assessment parts are complete. Feedback is available.' : 'Feedback is withheld until every assessment part is complete.'}</p>
      <button disabled={blocked || active !== null} onClick={() => void read(view.assessment.id)}>Refresh assessment</button>
      <ol>{view.assessment.parts.map((part,index) => <li key={part.sessionId}>{part.unitTitle}: {part.objective} — {view.parts[index]?.completed ?? 0}/{part.items.length}{view.parts[index]?.changedContent && <p>This lesson has changed; original questions are retained.</p>}<button disabled={blocked || active !== null} onClick={() => {
        if (view.parts[index]?.started) { setCanLeave(false); setActive(index); }
        else void startPart({ operationId: crypto.randomUUID(), sessionId: part.sessionId, expectedRevision: 0, action: { kind: 'startScopedAssessment', assessmentId: view.assessment.id, partIndex: index } }, index);
      }}>{view.parts[index]?.started ? 'Resume assessment part' : 'Start assessment part'}</button></li>)}</ol>
      {view.assessment.omissions.length > 0 && <p>{view.assessment.omissions.length} scope entries were excluded: instructional support, unsupported activities or lessons without assessable published content. These exclusions are retained with the assessment.</p>}
      <section aria-label="Assessment coverage"><h4>Coverage by outcome</h4><p>Declared unaided work, external help and unknown assistance remain separate. Recognition and self-reported practical work do not establish general competence.</p>
        {view.coverage.map(row => <article key={row.outcome}><h5>{outcomeLabel(row.outcome)}</h5><p>{row.attempted}/{row.total} responses; {row.unattempted} unattempted. No help declared: {row.declaredUnaided}; external help unknown: {row.externalUnknown}; external help declared: {row.externalPresent}.</p>
          {view.feedbackAvailable && row.feedback && <><p>Deterministic checks: {row.feedback.deterministicCorrect} correct, {row.feedback.deterministicIncorrect} incorrect, {row.feedback.deterministicUnknown} unknown.</p><p>Latest self-assessments: {row.feedback.selfCorrect} meets reference, {row.feedback.selfIncorrect} does not meet reference, {row.feedback.selfUnknown} unknown, {row.feedback.selfDisputed} disputed.</p></>}
        </article>)}
      </section>
    </>}
    {selected && view && <><StudyPanel key={selected.sessionId} unitId={selected.unitId} lessons={lessons} port={study} initialSessionId={selected.sessionId} onSafeToLeave={setCanLeave} /><button disabled={blocked || !canLeave} onClick={() => { setActive(null); void refresh(view.assessment.id).catch(reason => setError(failureMessage(reason))); }}>Return to assessment</button></>}
  </section>;
}
function outcomeLabel(value: string) { return ({ recall: 'Recall', recognition: 'Choice recognition', constructedResponse: 'Constructed response / problem solving', practicalSelfReport: 'Practical self-report' } as Record<string, string>)[value] ?? 'Unknown outcome'; }
