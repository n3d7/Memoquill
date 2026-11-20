import { useStudySettings } from './sessionSettings';
import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../application/errors';
import type { LessonPort } from '../lessons/types';
import type { ContinuityPort, ResumePage, ResumeSession } from './continuityTypes';
import type { ReviewOverview, ReviewPort } from './reviewTypes';
import type { StudyMutation, StudyPort } from './types';
import { StudyPanel } from './StudyPanel';

/** Start actions use durable canonical sessions and the existing recall policy. */
export function StudyStartPanel({ continuity, review, study, lessons, initialOverview }: { continuity: ContinuityPort; review: ReviewPort; study: StudyPort; lessons: LessonPort; initialOverview?: ReviewPort["overview"] }) {
  const newSettings = useStudySettings();
  const [resumable, setResumable] = useState<ResumePage | null>(null);
  const [due, setDue] = useState<ReviewOverview | null>(null);
  const [active, setActive] = useState<{ unitId: string; sessionId: string } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [pending, setPending] = useState<StudyMutation | null>(null);
  const [canLeave, setCanLeave] = useState(false);
  const epoch = useRef(0); const lock = useRef(false);
  const historyHeading = useRef<HTMLHeadingElement>(null);

  async function snapshot(offset = 0, focus = false, overview = () => review.overview()) {
    if (lock.current) return;
    lock.current = true; setBusy(true); setError(''); const token = epoch.current;
    try {
      const [saved, queue] = await Promise.all([continuity.sessions(offset), overview()]);
      if (token !== epoch.current) return;
      if (saved.contractVersion !== 1 || saved.offset !== offset) throw new Error('Resume page mismatch');
      setResumable(saved); setDue(queue);
      if (focus) historyHeading.current?.focus();
    } catch (reason) { if (token === epoch.current) setError(failureMessage(reason)); }
    finally { if (token === epoch.current) { lock.current = false; setBusy(false); } }
  }
  useEffect(() => {
    epoch.current++; lock.current = false; setResumable(null); setDue(null); setActive(null); setPending(null); setCanLeave(false); setError('');
    void snapshot(0, false, initialOverview);
    return () => { epoch.current++; lock.current = false; };
  }, [continuity, review, study, lessons, initialOverview]);

  async function resume(candidate: ResumeSession) {
    if (lock.current || pending || active) return;
    lock.current = true; setBusy(true); setError(''); const token = epoch.current;
    try {
      const saved = await study.read(candidate.sessionId);
      if (token !== epoch.current) return;
      if (saved.session.id !== candidate.sessionId || saved.session.unitId !== candidate.unitId || saved.session.cursor >= saved.session.items.length) throw new Error('The saved session changed. Refresh start actions.');
      setCanLeave(false); setActive({ unitId: candidate.unitId, sessionId: candidate.sessionId });
    } catch (reason) { if (token === epoch.current) setError(failureMessage(reason)); }
    finally { if (token === epoch.current) { lock.current = false; setBusy(false); } }
  }
  async function start(request: StudyMutation, retry = false) {
    if (lock.current || active || request.action.kind !== 'start') return;
    lock.current = true; setBusy(true); setError(''); setPending(request); const token = epoch.current;
    try {
      if (retry) await study.recover();
      if (token !== epoch.current) return;
      const saved = await study.apply(request);
      if (token !== epoch.current) return;
      if (saved.id !== request.sessionId || saved.unitId !== request.action.unitId) throw new Error('Session reply mismatch');
      setCanLeave(false); setActive({ unitId: saved.unitId, sessionId: saved.id }); setPending(null);
    } catch (reason) {
      if (token === epoch.current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) setPending(null); }
    } finally { if (token === epoch.current) { lock.current = false; setBusy(false); } }
  }
  const offered = due && !due.clockRollback ? due.offered.find(item => item.dueDay !== null && item.dueDay <= due.today) : undefined;
  const blocked = busy || pending !== null || active !== null;
  return <section aria-label="Continue learning"><h3>Continue learning</h3>
    {error && <p role="alert">{error} Refresh to check current availability; saved history is retained.</p>}
    {busy && <p role="status">Loading saved learning context…</p>}
    {pending && !busy && <button onClick={() => void start(pending, true)}>Reconnect and retry due session</button>}
    <button disabled={blocked} onClick={() => void snapshot()}>Refresh start actions</button>
    <button disabled={blocked || !resumable?.sessions.length || resumable.offset !== 0} onClick={() => { const session = resumable?.sessions[0]; if (session) void resume(session); }}>Resume saved session</button>
    <button disabled={blocked || !offered || !newSettings} onClick={() => { if (offered) void start({ operationId: crypto.randomUUID(), sessionId: crypto.randomUUID(), expectedRevision: 0, action: { kind: 'start', unitId: offered.unitId, compositionRevisionId: offered.compositionRevisionId, bindingIds: [offered.bindingId], mode: 'practice', settings: newSettings ?? undefined, phase: 'independent' } }); }}>Review due recall</button>
    {due && <p>{due.dueTotal} due recall items. {!offered && (due.clockRollback ? 'The clock rollback guard withholds reviews.' : 'No due recall is offered under the current policy and daily limits.')} Unscheduled new items are not substituted for due review.</p>}
    <h4 ref={historyHeading} tabIndex={-1}>Saved unfinished sessions</h4>
    {resumable && <><p role="status">{resumable.sessions.length} sessions shown from position {resumable.sessions.length ? resumable.offset + 1 : 0}. Most recently saved unfinished sessions appear first.</p>
      {!resumable.sessions.length && <p>No unfinished saved session on this page.</p>}
      <ul>{resumable.sessions.map(s => <li key={s.sessionId}>{s.title}: {s.cursor} of {s.total} submitted; {s.mode}; {s.savedResponse ? 'acknowledged response draft saved' : 'no response draft saved'}. {s.changedContent && 'Content changed; resume keeps the pinned study revision.'}<button disabled={blocked} onClick={() => void resume(s)}>Resume {s.title}</button></li>)}</ul>
      <nav aria-label="Saved session pages"><button disabled={blocked || resumable.offset === 0} onClick={() => void snapshot(Math.max(0, resumable.offset - 64), true)}>Previous saved sessions</button><button disabled={blocked || resumable.nextOffset === null} onClick={() => { if (resumable.nextOffset !== null) void snapshot(resumable.nextOffset, true); }}>Older saved sessions</button></nav>
    </>}
    {active && <><StudyPanel key={active.sessionId} unitId={active.unitId} lessons={lessons} port={study} initialSessionId={active.sessionId} onSafeToLeave={setCanLeave} /><button disabled={busy || pending !== null || !canLeave} onClick={() => { setActive(null); void snapshot(); }}>Return to start actions</button></>}
  </section>;
}
