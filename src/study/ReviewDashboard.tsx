import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../application/errors';
import type { LessonPort } from '../lessons/types';
import type { StudyMutation, StudyPort } from './types';
import type { RecallQueueItem, ReviewAction, ReviewMutation, ReviewOverview, ReviewPort } from './reviewTypes';
import { StudyPanel } from './StudyPanel';

export function ReviewDashboard({ port, study, lessons, units }: { port: ReviewPort; study: StudyPort; lessons: LessonPort; units: Record<string, { title: string; archived: boolean }> }) {
  const [view, setView] = useState<ReviewOverview | null>(null); const [error, setError] = useState(''); const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<ReviewMutation | null>(null); const [pendingStudy, setPendingStudy] = useState<StudyMutation | null>(null);
  const [canLeave, setCanLeave] = useState(false);
  const [active, setActive] = useState<{ unitId: string; sessionId: string } | null>(null);
  const [reminderUnit, setReminderUnit] = useState(''); const [reminderActivity, setReminderActivity] = useState(''); const [activities, setActivities] = useState<{ id: string; prompt: string }[]>([]);
  const [delay, setDelay] = useState('1'); const [interval, setInterval] = useState('');
  const lock = useRef(false); const mounted = useRef(true);
  async function refresh() { const value = await port.overview(); if (mounted.current) setView(value); }
  useEffect(() => { mounted.current = true; void refresh().catch(reason => { if (mounted.current) setError(failureMessage(reason)); }); return () => { mounted.current = false; }; }, [port]);
  useEffect(() => { let current = true; setActivities([]); setReminderActivity(''); if (reminderUnit) void lessons.read(reminderUnit).then(async saved => {
    const values = saved ? await Promise.all(saved.composition.bindings.filter(b => b.targetKind === 'activity').map(b => lessons.readActivity(reminderUnit, b.revisionId))) : [];
    if (current) setActivities(values.flatMap(v => v?.definition.family === 'procedure' ? [{ id: v.id, prompt: v.definition.prompt }] : []));
  }).catch(reason => { if (current) setError(failureMessage(reason)); }); return () => { current = false; }; }, [lessons, reminderUnit]);
  async function mutate(request: ReviewMutation, recover = false) {
    if (lock.current) return; lock.current = true; setBusy(true); setError(''); setPending(request);
    try { if (recover) await port.recover(); await port.apply(request); await refresh(); if (mounted.current) setPending(null); }
    catch (reason) { if (mounted.current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) setPending(null); } }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  function action(value: ReviewAction) { if (view && !busy && !pending && !pendingStudy) void mutate({ operationId: crypto.randomUUID(), expectedRevision: view.plan.revision, action: value }); }
  async function start(request: StudyMutation, unitId: string, recover = false) {
    if (lock.current) return; lock.current = true; setBusy(true); setError(''); setPendingStudy(request);
    try { if (recover) await study.recover(); await study.apply(request); if (mounted.current) { setCanLeave(false); setActive({ unitId, sessionId: request.sessionId }); setPendingStudy(null); } }
    catch (reason) { if (mounted.current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) setPendingStudy(null); } }
    finally { lock.current = false; if (mounted.current) setBusy(false); }
  }
  function review(item: RecallQueueItem) { if (!busy && !pending && !pendingStudy) void start({ operationId: crypto.randomUUID(), sessionId: crypto.randomUUID(), expectedRevision: 0, action: { kind: 'start', unitId: item.unitId, compositionRevisionId: item.compositionRevisionId, bindingIds: [item.bindingId], mode: 'practice', phase: 'independent' } }, item.unitId); }
  const blocked = busy || pending !== null || pendingStudy !== null;
  return <section aria-label="Recall review and practical reminders"><h3>Recall review</h3>
    <p>FSRS schedules explicitly rated repeatable recall. It does not estimate problem-solving or practical competence. Practical reminders are separate.</p>
    {error && <p role="alert">{error}</p>}{busy && <p role="status">Saving or loading review state…</p>}
    {pending && !busy && <button onClick={() => void mutate(pending, true)}>Reconnect and retry review change</button>}
    {pendingStudy && !busy && pendingStudy.action.kind === 'start' && <button onClick={() => void start(pendingStudy, pendingStudy.action.kind === 'start' ? pendingStudy.action.unitId : '', true)}>Reconnect and retry recall session</button>}
    {view && <><p>Study day {view.todayDate}, {view.plan.timeZone}, starts at {view.plan.dayStartHour}:00. New {view.newUsed}/{view.plan.dailyNewLimit}; reviews {view.reviewsUsed}/{view.plan.dailyReviewLimit}. Overdue/due {view.dueTotal}; unscheduled recall candidates {view.newTotal}.</p>
      {view.clockRollback && <p role="alert">The clock is earlier than your latest review. Scheduled reviews are withheld until it is corrected. History is retained.</p>}
      <button disabled={blocked} onClick={() => { void refresh().catch(reason => setError(failureMessage(reason))); }}>Refresh recall queue</button>
      <ul>{view.offered.map(item => <li key={item.activityRevisionId}>{units[item.unitId]?.title ?? 'Lesson'}: {item.prompt} — {item.dueDate ?? 'Not scheduled'} <button disabled={blocked || active !== null} onClick={() => review(item)}>Review this recall</button></li>)}</ul>
      <p>Limits defer work without deleting overdue items. New content revisions start separate schedules. Policy changes apply to subsequent reviews and retain previous events.</p>
      <Policy key={view.plan.revision} view={view} disabled={blocked} save={action} />
      <section aria-label="Practical reminders"><h4>Practical reminders</h4><p>Completion is a self-reported reminder action; it does not create a study attempt or prove competence.</p>
        <ul>{view.plan.reminders.filter(r => !r.archived).map(r => <li key={r.id}>{units[r.unitId]?.title ?? 'Lesson'}: {r.dueDay <= view.today ? 'Due' : `In ${r.dueDay - view.today} study days`}; {r.intervalDays ? `every ${r.intervalDays} days` : 'manual reminder'} <button disabled={blocked} onClick={() => action({ kind: 'completeReminder', id: r.id })}>Mark reminder completed</button><button disabled={blocked} onClick={() => action({ kind: 'archiveReminder', id: r.id })}>Archive reminder</button></li>)}</ul>
        <form onSubmit={e => { e.preventDefault(); action({ kind: 'addReminder', id: crypto.randomUUID(), unitId: reminderUnit, activityRevisionId: reminderActivity, dueDay: view.today + Number(delay), intervalDays: interval ? Number(interval) : null }); }}><fieldset disabled={blocked}><legend>Add a practical reminder</legend>
          <label>Reminder lesson<select value={reminderUnit} onChange={e => setReminderUnit(e.target.value)}><option value="">Choose lesson</option>{Object.entries(units).filter(([,u]) => !u.archived).map(([id,u]) => <option key={id} value={id}>{u.title}</option>)}</select></label>
          <label>Practical activity<select value={reminderActivity} onChange={e => setReminderActivity(e.target.value)}><option value="">Choose published procedure</option>{activities.map(a => <option key={a.id} value={a.id}>{a.prompt}</option>)}</select></label>
          <label>Due in study days<input type="number" min="0" max="3650" required value={delay} onChange={e => setDelay(e.target.value)} /></label><label>Repeat every days (empty for manual)<input type="number" min="1" max="3650" value={interval} onChange={e => setInterval(e.target.value)} /></label><button disabled={!reminderActivity}>Add reminder</button>
        </fieldset></form>
      </section>
    </>}
    {active && <><p>Finish or save this session before returning to the queue. Saved sessions also remain available in the lesson.</p><StudyPanel key={active.sessionId} unitId={active.unitId} lessons={lessons} port={study} initialSessionId={active.sessionId} onSafeToLeave={setCanLeave} /><button disabled={blocked || !canLeave} onClick={() => { setActive(null); void refresh().catch(reason => setError(failureMessage(reason))); }}>Return to recall queue</button></>}
  </section>;
}
function Policy({ view, disabled, save }: { view: ReviewOverview; disabled: boolean; save: (action: ReviewAction) => void }) {
  const [zone, setZone] = useState(view.plan.timeZone); const [hour, setHour] = useState(String(view.plan.dayStartHour)); const [newLimit, setNewLimit] = useState(String(view.plan.dailyNewLimit)); const [reviewLimit, setReviewLimit] = useState(String(view.plan.dailyReviewLimit)); const [retention, setRetention] = useState(String(view.plan.retentionBasisPoints / 100));
  return <form onSubmit={e => { e.preventDefault(); save({ kind: 'setPolicy', timeZone: zone, dayStartHour: Number(hour), dailyNewLimit: Number(newLimit), dailyReviewLimit: Number(reviewLimit), retentionBasisPoints: Math.round(Number(retention) * 100) }); }}><fieldset disabled={disabled}><legend>Recall policy</legend>
    <label>IANA time zone<input required value={zone} maxLength={128} onChange={e => setZone(e.target.value)} /></label><label>Study day starts at hour<input type="number" min="0" max="23" required value={hour} onChange={e => setHour(e.target.value)} /></label>
    <label>Daily new recall limit<input type="number" min="0" max="100" required value={newLimit} onChange={e => setNewLimit(e.target.value)} /></label><label>Daily review limit<input type="number" min="0" max="500" required value={reviewLimit} onChange={e => setReviewLimit(e.target.value)} /></label>
    <label>Target recall retention (%)<input type="number" min="70" max="99" step="0.1" required value={retention} onChange={e => setRetention(e.target.value)} /></label><button>Save recall policy</button>
  </fieldset></form>;
}
