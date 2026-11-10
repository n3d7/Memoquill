import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../contentBridge';
import type { OriginalSource } from '../application/sourceTypes';
import type { JsonPropertyMapping, ApplyJob, ApplyReimport, CreateJob, Decision, Disposition, IngestionJob, IngestionPort, JobAction, JobSummary, Mapping, ReimportPreview, ReviewJob, UndoJob, UndoPreview } from '../application/ingestionTypes';

type Pending = { kind: 'create'; request: CreateJob } | { kind: 'apply'; request: ApplyJob } | { kind: 'review'; request: ReviewJob } | { kind: 'reimport'; request: ApplyReimport } | { kind: 'cancel'; request: JobAction } | { kind: 'undo'; request: UndoJob };
const defaultMapping: Mapping = { keyColumn: 0, titleColumn: 0, bodyColumn: 0 };
const rawJsonProperties: JsonPropertyMapping = { key: null, title: null, body: null };
const rawJsonMapping: Mapping = { ...defaultMapping, jsonProperties: rawJsonProperties };
const initialMapping = (job: IngestionJob) => job.mapping ?? (job.handlerVersion === 2 && job.extraction.format === 'json' ? rawJsonMapping : defaultMapping);
function selectedValues(record: IngestionJob['extraction']['records'][number], mapping: Mapping) {
  if (record.duplicateProperties?.length) return null;
  const value = (property: string | null) => property === null ? record.fields[0] : record.properties && Object.hasOwn(record.properties, property) ? record.properties[property] : undefined;
  const fields = mapping.jsonProperties;
  if (!fields) {
    const [key, title, body] = [record.fields[mapping.keyColumn], record.fields[mapping.titleColumn], record.fields[mapping.bodyColumn]];
    return key === undefined || title === undefined || body === undefined ? null : { key, title, body };
  }
  const key = value(fields.key), title = value(fields.title), body = value(fields.body);
  return key === undefined || title === undefined || body === undefined ? null : { key, title, body };
}
export function IngestionPanel({ port, sources, workspaceId, resourceId, libraryRevision, onChanged, onPendingChange }: {
  port: IngestionPort; sources: OriginalSource[]; workspaceId: string; resourceId: string; libraryRevision: number;
  onChanged: () => void; onPendingChange?: (pending: boolean) => void;
}) {
  const [sourceId, setSourceId] = useState(''); const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [job, setJob] = useState<IngestionJob | null>(null); const [mapping, setMapping] = useState<Mapping>(defaultMapping);
  const [selected, setSelected] = useState<number[]>([]); const [page, setPage] = useState(0);
  const [hasMoreJobs, setHasMoreJobs] = useState(false); const [listing, setListing] = useState(false); const loadedPages = useRef(1);
  const comparisonGeneration = useRef(0);
  const [overwrites, setOverwrites] = useState<number[]>([]);
  const [disposition, setDisposition] = useState<Disposition>('omitted');
  const [previousId, setPreviousId] = useState(''); const [preview, setPreview] = useState<ReimportPreview | null>(null); const [previousRevision, setPreviousRevision] = useState<number | null>(null);
  const [undoPreview, setUndoPreview] = useState<UndoPreview | null>(null);
  const [pending, setPending] = useState<Pending | null>(null); const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const [status, setStatus] = useState('');
  const generation = useRef(0);
  function invalidateComparison() { comparisonGeneration.current++; setPreview(null); setUndoPreview(null); setPreviousRevision(null); setSelected([]); setOverwrites([]); }
  useEffect(() => {
    const current = ++generation.current;
    setJob(null); setJobs([]); setMapping(defaultMapping); setPage(0); setPreviousId(''); invalidateComparison();
    setPending(null); setBusy(false); setError(''); setStatus(''); setSourceId(''); setHasMoreJobs(false); setListing(true); loadedPages.current = 1;
    port.list(resourceId, 0).then(value => { if (generation.current === current) { setJobs(value); setHasMoreJobs(value.length === 20); } }, () => { if (generation.current === current) setError('Import jobs could not be loaded.'); }).finally(() => { if (generation.current === current) setListing(false); });
    return () => { generation.current++; };
  }, [port, resourceId, workspaceId]);
  useEffect(() => { invalidateComparison(); }, [libraryRevision]);
  async function loadMoreJobs() {
    const current = generation.current; const offset = loadedPages.current * 20; setListing(true); setError('');
    try { const value = await port.list(resourceId, offset); if (generation.current === current) { setJobs(previous => [...previous, ...value.filter(item => !previous.some(old => old.id === item.id))]); loadedPages.current++; setHasMoreJobs(value.length === 20); } }
    catch (reason) { if (generation.current === current) setError(failureMessage(reason)); }
    finally { if (generation.current === current) setListing(false); }
  }
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
      setJob(result); setMapping(initialMapping(result)); setPending(null); invalidateComparison(); setPreviousId(result.previousJobId ?? ''); setPage(0); setStatus('Import operation saved locally.');
      if (action.kind === 'apply' || action.kind === 'reimport' || action.kind === 'undo') onChanged();
      void Promise.all(Array.from({ length: loadedPages.current }, (_, page) => port.list(resourceId, page * 20))).then(pages => { if (generation.current === current) { setJobs(pages.flat()); setHasMoreJobs(pages.at(-1)?.length === 20); } }, () => { if (generation.current === current) setError('Import jobs could not be refreshed.'); });
    } catch (reason) { if (generation.current === current) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) { setPending(null); invalidateComparison(); } } }
    finally { if (generation.current === current) setBusy(false); }
  }
  const envelope = () => ({ contractVersion: 1 as const, workspaceId, operationId: crypto.randomUUID() });
  function create() { const source = sources.find(x => x.id === sourceId) ?? sources[0]; if (source) void send({ kind: 'create', request: { ...envelope(), sourceId: source.id, sourceRevision: source.revision } }); }
  async function open(id: string) { const current = generation.current; setBusy(true); setError(''); try { const next = await port.read(id); if (generation.current === current) { setJob(next); setMapping(initialMapping(next)); invalidateComparison(); setPreviousId(next.previousJobId ?? ''); setPreviousRevision(null); setPage(0); } } catch (reason) { if (generation.current === current) setError(failureMessage(reason)); } finally { if (generation.current === current) setBusy(false); } }
  async function compare() { if (!job || !previousId) return; setBusy(true); setError(''); invalidateComparison(); const current = generation.current; const comparison = comparisonGeneration.current; try { const [value, previous] = await Promise.all([port.reimportPreview(job.id, previousId, mapping), port.read(previousId)]); if (generation.current === current && comparisonGeneration.current === comparison) { setPreview(value); setPreviousRevision(previous.revision); setSelected([]); setOverwrites([]); } } catch (reason) { if (generation.current === current && comparisonGeneration.current === comparison) setError(failureMessage(reason)); } finally { if (generation.current === current) setBusy(false); } }
  async function inspectUndo() { if (!job) return; setBusy(true); setError(''); setUndoPreview(null); const current = generation.current; const comparison = comparisonGeneration.current; try { const value = await port.undoPreview(job.id); if (generation.current === current && comparisonGeneration.current === comparison) setUndoPreview(value); } catch (reason) { if (generation.current === current && comparisonGeneration.current === comparison) setError(failureMessage(reason)); } finally { if (generation.current === current) setBusy(false); } }
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
  const locked = busy || pending !== null; const records = job?.extraction.records ?? [];
  const keyCounts = new Map<string, number>();
  for (const record of records) { const key = selectedValues(record, mapping)?.key.trim(); if (key) keyCounts.set(key, (keyCounts.get(key) ?? 0) + 1); }
  const reviewedCount = job?.decisions?.filter(item => item.index !== null).length ?? 0;
  const visible = records.slice(page * 50, page * 50 + 50);
  return <section aria-label="Import sources"><h3>Import sources</h3>
    <p>Review saved records and select only those to import. Columns are numbered from 1; the first CSV row is data.</p>
    {error && <p role="alert">{error}</p>}{status && <p role="status">{status}</p>}
    {pending && !busy && <button onClick={() => void send(pending)}>Retry same import</button>}
    <fieldset disabled={locked}><label>Saved source<select value={sourceId} onChange={event => setSourceId(event.target.value)}>{sources.map(source => <option key={source.id} value={source.id}>{source.filename}</option>)}</select></label>
      <button disabled={!sources.length} onClick={create}>Create import preview</button></fieldset>
    <ul>{jobs.map(item => <li key={item.id}><button disabled={locked} onClick={() => void open(item.id)}>Open import {item.id}</button> {item.status}, {item.appliedCount}/{item.recordCount} applied</li>)}</ul>
    <button disabled={locked || listing || !hasMoreJobs} onClick={() => void loadMoreJobs()}>Load more import jobs</button>
    {job && <fieldset disabled={locked}><legend>Import {job.source.filename}</legend>
      <p>Extraction: {job.extraction.status}{job.extraction.diagnostic ? ` (${job.extraction.diagnostic})` : ''}. {records.length} records.</p>
      <p>Outcomes: {job.outcomes.length} applied, {reviewedCount} reviewed, {records.length - job.outcomes.length - reviewedCount} pending.</p>
      {job.handlerVersion === 2 && job.extraction.format === 'json' ? <>
        <p>Choose first-level JSON properties by name or the exact raw record. Missing selected properties and duplicate names remain unresolved.</p>
        <label><input type="checkbox" disabled={job.mapping !== null} checked={!mapping.jsonProperties} onChange={event => { setMapping(event.target.checked ? defaultMapping : rawJsonMapping); invalidateComparison(); }} />Use legacy raw JSON mapping</label>
        {(['key', 'title', 'body'] as const).map((key, index) => <label key={key}>{['Key property', 'Title property', 'Body property'][index]}<select disabled={job.mapping !== null} value={mapping.jsonProperties?.[key] == null ? 'raw:' : `property:${mapping.jsonProperties?.[key] ?? ''}`} onChange={event => { const property = event.target.value === 'raw:' ? null : event.target.value.slice(9); setMapping({ ...mapping, jsonProperties: { ...(mapping.jsonProperties ?? rawJsonProperties), [key]: property } }); invalidateComparison(); }}><option value="raw:">Raw JSON record</option>{Array.from(new Set([...records.flatMap(record => [...Object.keys(record.properties ?? {}), ...(record.duplicateProperties ?? [])]), ...Object.values(mapping.jsonProperties ?? {}).filter((name): name is string => name !== null)])).sort().map(name => <option key={name} value={`property:${name}`}>{JSON.stringify(name)}</option>)}</select></label>)}
      </> : (['keyColumn', 'titleColumn', 'bodyColumn'] as const).map((key, index) => <label key={key}>{['Key column', 'Title column', 'Body column'][index]}<input disabled={job.mapping !== null} type="number" min="1" max="64" value={mapping[key] + 1} onChange={event => { setMapping({ ...mapping, [key]: Math.max(0, Number(event.target.value) - 1) }); invalidateComparison(); }} /></label>)}
      <label>Reimport from<select disabled={job.previousJobId !== null} value={previousId} onChange={event => { setPreviousId(event.target.value); invalidateComparison(); }}><option value="">New import</option>{jobs.filter(item => item.id !== job.id && (item.eligibleBaseline || item.id === job.previousJobId)).map(item => <option key={item.id} value={item.id}>{item.id}</option>)}</select></label>
      {previousId && <button onClick={() => void compare()}>Compare reimport</button>}
      {preview && <ul>{preview.items.map((item, index) => <li key={`${item.key}-${index}`}>{item.key || '(unresolved)'}: {item.status}{item.prior && <div role="group" aria-label={`Prior imported values for ${item.key}`}><p>Prior title: {item.prior.title}</p><pre>{item.prior.body}</pre><p>{item.prior.locator.kind}: {item.prior.locator.value || '(root)'}</p></div>}{item.currentBody != null && <div role="group" aria-label={`Current human values for ${item.key}`}><p>Current title: {item.currentTitle}</p><p>Current revision: {item.currentRevisionId}</p><pre>{item.currentBody}</pre></div>}{item.incoming && <div role="group" aria-label={`New source values for ${item.key}`}><p>New title: {item.incoming.title}</p><pre>{item.incoming.body}</pre><p>{item.incoming.locator.kind}: {item.incoming.locator.value || '(root)'}</p></div>}{item.status === 'removed' && <button onClick={() => review([{ index: null, key: item.key, disposition: job.decisions?.some(x => x.key === item.key && x.disposition === 'removedRetained') ? 'pending' : 'removedRetained' }])}>{job.decisions?.some(x => x.key === item.key && x.disposition === 'removedRetained') ? 'Reopen removed' : 'Retain removed'} {item.key}</button>}{item.status === 'conflict' && item.index !== null && <label><input type="checkbox" checked={overwrites.includes(item.index)} onChange={event => { const record = item.index!; setOverwrites(event.target.checked ? [...overwrites, record] : overwrites.filter(x => x !== record)); if (!event.target.checked) setSelected(value => value.filter(x => x !== record)); }} />Explicitly replace current human edits for {item.key} (current title: {item.currentTitle}; revision: {item.currentRevisionId})</label>}</li>)}</ul>}
      <ul>{visible.map((record, index) => { const absolute = page * 50 + index; const applied = job.outcomes.some(x => x.index === absolute); const decision = job.decisions?.find(x => x.index === absolute); const comparison = preview?.items.find(item => item.index === absolute); const values = selectedValues(record, mapping); const unresolved = !values || !values.key.trim() || !values.title.trim() || (keyCounts.get(values.key.trim()) ?? 0) > 1; return <li key={absolute}><label><input type="checkbox" aria-label={`Select record ${absolute + 1}`} checked={selected.includes(absolute)} disabled={unresolved || applied || !!decision || (previousId !== '' && (!comparison || (!['new', 'changed', 'unchanged'].includes(comparison.status) && !(comparison.status === 'conflict' && overwrites.includes(absolute)))))} onChange={event => setSelected(event.target.checked ? [...selected, absolute] : selected.filter(x => x !== absolute))} />Record {absolute + 1}: {record.fields.join(' | ')}</label>{job.handlerVersion === 2 && <><p>Original JSON pointer: {record.locator.value || '(root)'}</p>{!!record.duplicateProperties?.length ? <p>Duplicate JSON properties: {record.duplicateProperties.join(', ')} — unresolved</p> : values ? <p>Selected values — key: {values.key}; title: {values.title}; body: {values.body}</p> : <p>Missing selected JSON property — unresolved</p>}</>}{!applied && !decision && ` — ${job.status === 'staged' ? unresolved ? 'unresolved; pending review' : 'pending review' : 'not applied; job ' + job.status}`}{applied && ' — applied'}{decision && ` — ${decision.disposition}`}{!applied && <button onClick={() => review([{ index: absolute, key: null, disposition: decision ? 'pending' : record.fields.every(field => !field.trim()) ? 'empty' : unresolved ? 'unresolved' : comparison?.status === 'unresolved' ? 'unresolved' : 'omitted' }])}>{decision ? `Reopen record ${absolute + 1}` : `Exclude record ${absolute + 1}`}</button>}</li>; })}</ul>
      <button disabled={page === 0} onClick={() => setPage(page - 1)}>Previous records</button><button disabled={(page + 1) * 50 >= records.length} onClick={() => setPage(page + 1)}>Next records</button>
      <label>Selected record outcome<select value={disposition} onChange={event => setDisposition(event.target.value as Disposition)}><option value="omitted">Omitted</option><option value="empty">Empty</option><option value="unresolved">Unresolved</option></select></label>
      <button disabled={job.status !== 'staged' || !selected.length || selected.length > 100 || (!!previousId && !preview)} onClick={() => review(selected.map(index => ({ index, key: null, disposition })))}>Record selected outcomes</button>
      <button disabled={job.status !== 'staged' || job.extraction.status !== 'complete' || !selected.length || selected.length > 100 || (!!previousId && (!preview || previousRevision === null))} onClick={apply}>Apply selected records</button>
      <button disabled={job.status !== 'staged'} onClick={() => void send({ kind: 'cancel', request: { ...envelope(), jobId: job.id, expectedRevision: job.revision } })}>Cancel remaining import</button>
      <button onClick={() => void inspectUndo()}>Preview undo</button>{undoPreview && <p>{undoPreview.canUndo ? `Undo will compensate ${undoPreview.resources.length} unchanged imported materials.` : 'Undo is unavailable because content or dependencies changed.'}</p>}
      {undoPreview && <ul>{undoPreview.diagnostics?.map((message, index) => <li key={index}>{message}</li>)}</ul>}
      {undoPreview?.canUndo && <button onClick={() => void send({ kind: 'undo', request: { ...envelope(), jobId: job.id, expectedRevision: job.revision, expectedLibraryRevision: libraryRevision } })}>Confirm undo</button>}
    </fieldset>}
  </section>;
}
