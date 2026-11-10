import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../contentBridge';
import type { OriginalSource } from '../application/sourceTypes';
import type { ApplyJob, ApplyReimport, CreateJob, Decision, Disposition, IngestionJob, IngestionPort, JobAction, JobSummary, Mapping, ReimportPreview, ReviewJob, UndoJob, UndoPreview } from '../application/ingestionTypes';

type Pending = { kind: 'create'; request: CreateJob } | { kind: 'apply'; request: ApplyJob } | { kind: 'review'; request: ReviewJob } | { kind: 'reimport'; request: ApplyReimport } | { kind: 'cancel'; request: JobAction } | { kind: 'undo'; request: UndoJob };
const defaultMapping: Mapping = { keyColumn: 0, titleColumn: 0, bodyColumn: 0 };
export function IngestionPanel({ port, sources, workspaceId, resourceId, libraryRevision, onChanged, onPendingChange }: {
  port: IngestionPort; sources: OriginalSource[]; workspaceId: string; resourceId: string; libraryRevision: number;
  onChanged: () => void; onPendingChange?: (pending: boolean) => void;
}) {
  const [sourceId, setSourceId] = useState(''); const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [job, setJob] = useState<IngestionJob | null>(null); const [mapping, setMapping] = useState<Mapping>(defaultMapping);
  const [selected, setSelected] = useState<number[]>([]); const [page, setPage] = useState(0);
  const [overwrites, setOverwrites] = useState<number[]>([]);
  const [disposition, setDisposition] = useState<Disposition>('omitted');
  const [previousId, setPreviousId] = useState(''); const [preview, setPreview] = useState<ReimportPreview | null>(null); const [previousRevision, setPreviousRevision] = useState<number | null>(null);
  const [undoPreview, setUndoPreview] = useState<UndoPreview | null>(null);
  const [pending, setPending] = useState<Pending | null>(null); const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const [status, setStatus] = useState('');
  const generation = useRef(0);
  useEffect(() => { const current = ++generation.current; setJob(null); setSelected([]); setPending(null); setPreview(null); setUndoPreview(null); setSourceId(''); port.list(resourceId, 0).then(value => { if (generation.current === current) setJobs(value); }, () => { if (generation.current === current) setError('Import jobs could not be loaded.'); }); return () => { generation.current++; }; }, [port, resourceId, workspaceId]);
  useEffect(() => { onPendingChange?.(pending !== null || busy); return () => onPendingChange?.(false); }, [pending, busy, onPendingChange]);
  async function send(action: Pending) {
    const current = generation.current; setPending(action); setBusy(true); setError(''); setStatus('');
    try {
      let result: IngestionJob;
      if (action.kind === 'create') result = await port.create(action.request);
      else if (action.kind === 'apply') result = await port.apply(action.request);
      else if (action.kind === 'review') result = await port.review(action.request);
      else if (action.kind === 'reimport') result = await port.reimportApply(action.request);
      else if (action.kind === 'cancel') result = await port.cancel(action.request);
      else result = await port.undo(action.request);
      if (generation.current !== current) return;
      setJob(result); setPending(null); setSelected([]); setPreview(null); setUndoPreview(null); setStatus('Import operation saved locally.');
      if (action.kind === 'apply' || action.kind === 'reimport' || action.kind === 'undo') onChanged();
      void port.list(resourceId, 0).then(value => { if (generation.current === current) setJobs(value); }, () => { if (generation.current === current) setError('Import jobs could not be refreshed.'); });
    } catch (reason) { if (generation.current === current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) setPending(null); } }
    finally { if (generation.current === current) setBusy(false); }
  }
  const envelope = () => ({ contractVersion: 1 as const, workspaceId, operationId: crypto.randomUUID() });
  function create() { const source = sources.find(x => x.id === sourceId) ?? sources[0]; if (source) void send({ kind: 'create', request: { ...envelope(), sourceId: source.id, sourceRevision: source.revision } }); }
  async function open(id: string) { const current = generation.current; setBusy(true); setError(''); try { const next = await port.read(id); if (generation.current === current) { setJob(next); setMapping(next.mapping ?? defaultMapping); setSelected([]); setPreview(null); setUndoPreview(null); setPreviousId(next.previousJobId ?? ''); setPreviousRevision(null); setPage(0); } } catch (reason) { if (generation.current === current) setError(failureMessage(reason)); } finally { if (generation.current === current) setBusy(false); } }
  async function compare() { if (!job || !previousId) return; setBusy(true); setError(''); const current = generation.current; try { const [value, previous] = await Promise.all([port.reimportPreview(job.id, previousId, mapping), port.read(previousId)]); if (generation.current === current) { setPreview(value); setPreviousRevision(previous.revision); setSelected([]); setOverwrites([]); } } catch (reason) { if (generation.current === current) setError(failureMessage(reason)); } finally { if (generation.current === current) setBusy(false); } }
  async function inspectUndo() { if (!job) return; setBusy(true); setError(''); const current = generation.current; try { const value = await port.undoPreview(job.id); if (generation.current === current) setUndoPreview(value); } catch (reason) { if (generation.current === current) setError(failureMessage(reason)); } finally { if (generation.current === current) setBusy(false); } }
  function apply() {
    if (!job || selected.length === 0) return;
    const common = { ...envelope(), jobId: job.id, expectedRevision: job.revision, expectedLibraryRevision: libraryRevision, mapping, selected };
    if (previousId && preview && previousRevision !== null) {
      const resolutions = preview.items.filter(item => item.status === 'conflict' && item.index !== null && selected.includes(item.index) && overwrites.includes(item.index) && item.currentRevisionId !== null && item.currentTitle !== null).map(item => ({ index: item.index!, expectedDocumentRevisionId: item.currentRevisionId!, expectedTitle: item.currentTitle! }));
      void send({ kind: 'reimport', request: { ...common, previousJobId: previousId, expectedPreviousRevision: previousRevision, resolutions } });
    }
    else if (!previousId) void send({ kind: 'apply', request: common });
  }
  function review(decisions: Decision[]) {
    if (!job || (previousId !== '' && (!preview || previousRevision === null))) return;
    void send({ kind: 'review', request: { ...envelope(), jobId: job.id, expectedRevision: job.revision, previousJobId: previousId || null, expectedPreviousRevision: previousId ? previousRevision : null, mapping, decisions } });
  }
  const locked = busy || pending !== null; const records = job?.extraction.records ?? []; const visible = records.slice(page * 50, page * 50 + 50);
  return <section aria-label="Import sources"><h3>Import sources</h3>
    <p>Review saved records and select only those to import. Columns are numbered from 1; the first CSV row is data.</p>
    {error && <p role="alert">{error}</p>}{status && <p role="status">{status}</p>}
    {pending && !busy && <button onClick={() => void send(pending)}>Retry same import</button>}
    <fieldset disabled={locked}><label>Saved source<select value={sourceId} onChange={event => setSourceId(event.target.value)}>{sources.map(source => <option key={source.id} value={source.id}>{source.filename}</option>)}</select></label>
      <button disabled={!sources.length} onClick={create}>Create import preview</button></fieldset>
    <ul>{jobs.map(item => <li key={item.id}><button disabled={locked} onClick={() => void open(item.id)}>Open import {item.id}</button> {item.status}, {item.appliedCount}/{item.recordCount} applied</li>)}</ul>
    {job && <fieldset disabled={locked}><legend>Import {job.source.filename}</legend>
      <p>Extraction: {job.extraction.status}{job.extraction.diagnostic ? ` (${job.extraction.diagnostic})` : ''}. {records.length} records.</p>
      {(['keyColumn', 'titleColumn', 'bodyColumn'] as const).map((key, index) => <label key={key}>{['Key column', 'Title column', 'Body column'][index]}<input disabled={job.mapping !== null} type="number" min="1" max="64" value={mapping[key] + 1} onChange={event => { setMapping({ ...mapping, [key]: Math.max(0, Number(event.target.value) - 1) }); setPreview(null); setSelected([]); setOverwrites([]); }} /></label>)}
      <label>Reimport from<select disabled={job.previousJobId !== null} value={previousId} onChange={event => { setPreviousId(event.target.value); setPreview(null); setSelected([]); setOverwrites([]); }}><option value="">New import</option>{jobs.filter(item => item.id !== job.id && item.appliedCount > 0 && item.status !== 'undone').map(item => <option key={item.id} value={item.id}>{item.id}</option>)}</select></label>
      {previousId && <button onClick={() => void compare()}>Compare reimport</button>}
      {preview && <ul>{preview.items.map((item, index) => <li key={`${item.key}-${index}`}>{item.key || '(unresolved)'}: {item.status}{item.status === 'removed' && <button onClick={() => review([{ index: null, key: item.key, disposition: job.decisions?.some(x => x.key === item.key && x.disposition === 'removedRetained') ? 'pending' : 'removedRetained' }])}>{job.decisions?.some(x => x.key === item.key && x.disposition === 'removedRetained') ? 'Reopen removed' : 'Retain removed'} {item.key}</button>}{item.status === 'conflict' && item.index !== null && <label><input type="checkbox" checked={overwrites.includes(item.index)} onChange={event => { const record = item.index!; setOverwrites(event.target.checked ? [...overwrites, record] : overwrites.filter(x => x !== record)); if (!event.target.checked) setSelected(value => value.filter(x => x !== record)); }} />Explicitly replace current human edits for {item.key} (current title: {item.currentTitle})</label>}</li>)}</ul>}
      <ul>{visible.map((record, index) => { const absolute = page * 50 + index; const applied = job.outcomes.some(x => x.index === absolute); const decision = job.decisions?.find(x => x.index === absolute); const comparison = preview?.items.find(item => item.index === absolute); return <li key={absolute}><label><input type="checkbox" aria-label={`Select record ${absolute + 1}`} checked={selected.includes(absolute)} disabled={applied || !!decision || (previousId !== '' && (!comparison || (!['new', 'changed', 'unchanged'].includes(comparison.status) && !(comparison.status === 'conflict' && overwrites.includes(absolute)))))} onChange={event => setSelected(event.target.checked ? [...selected, absolute] : selected.filter(x => x !== absolute))} />Record {absolute + 1}: {record.fields.join(' | ')}</label>{applied && ' — applied'}{decision && ` — ${decision.disposition}`}{!applied && <button onClick={() => review([{ index: absolute, key: null, disposition: decision ? 'pending' : record.fields.every(field => !field.trim()) ? 'empty' : comparison?.status === 'unresolved' ? 'unresolved' : 'omitted' }])}>{decision ? `Reopen record ${absolute + 1}` : `Exclude record ${absolute + 1}`}</button>}</li>; })}</ul>
      <button disabled={page === 0} onClick={() => setPage(page - 1)}>Previous records</button><button disabled={(page + 1) * 50 >= records.length} onClick={() => setPage(page + 1)}>Next records</button>
      <label>Selected record outcome<select value={disposition} onChange={event => setDisposition(event.target.value as Disposition)}><option value="omitted">Omitted</option><option value="empty">Empty</option><option value="unresolved">Unresolved</option></select></label>
      <button disabled={job.status !== 'staged' || !selected.length || selected.length > 100 || (!!previousId && !preview)} onClick={() => review(selected.map(index => ({ index, key: null, disposition })))}>Record selected outcomes</button>
      <button disabled={job.status !== 'staged' || job.extraction.status !== 'complete' || !selected.length || selected.length > 100 || (!!previousId && (!preview || previousRevision === null))} onClick={apply}>Apply selected records</button>
      <button disabled={job.status !== 'staged'} onClick={() => void send({ kind: 'cancel', request: { ...envelope(), jobId: job.id, expectedRevision: job.revision } })}>Cancel remaining import</button>
      <button onClick={() => void inspectUndo()}>Preview undo</button>{undoPreview && <p>{undoPreview.canUndo ? `Undo will compensate ${undoPreview.resources.length} unchanged imported materials.` : 'Undo is unavailable because content or dependencies changed.'}</p>}
      {undoPreview?.canUndo && <button onClick={() => void send({ kind: 'undo', request: { ...envelope(), jobId: job.id, expectedRevision: job.revision, expectedLibraryRevision: libraryRevision } })}>Confirm undo</button>}
    </fieldset>}
  </section>;
}
