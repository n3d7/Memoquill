import { useEffect, useRef, useState } from 'react';
import type { Dispatch, RefObject, SetStateAction } from 'react';
import { failureMessage } from './contentBridge';
import { useSourcePort } from './application/ApplicationProvider';
import type { SourcePort } from './application/ports';
import type { OriginalSource, SaveSource, SelectedSource, SourceList, SourcePreview } from './sourceBridge';
import { usePickerFocus } from './usePickerFocus';

interface Draft { selection: SelectedSource; attribution: string; license: string; originUrl: string }
interface AttachmentState {
  draft: Draft | null; pending: SaveSource | null; busy: 'idle' | 'picking' | 'saving'; error: string; status: string;
}
interface PreviewState { preview: SourcePreview | null; busy: boolean; error: string }
interface Session<T> { state: T; setState: Dispatch<SetStateAction<T>>; generation: RefObject<number> }
const emptyAttachment: AttachmentState = { draft: null, pending: null, busy: 'idle', error: '', status: '' };
const emptyPreview: PreviewState = { preview: null, busy: false, error: '' };

function useSourceSession<T>(workspaceId: string, resourceId: string, initial: T): Session<T> {
  const [state, setState] = useState(initial);
  const generation = useRef(0);
  useEffect(() => {
    generation.current++;
    setState(initial);
    return () => { generation.current++; };
  }, [workspaceId, resourceId, initial]);
  return { state, setState, generation };
}

function useSourceList(port: SourcePort, workspaceId: string, resourceId: string) {
  const [list, setList] = useState<SourceList | null>(null);
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    setList(null); setError('');
    port.sources(resourceId).then(value => {
      if (active) setList(value);
    }, failure => {
      if (active) setError(`Sources unavailable. ${failureMessage(failure)}`);
    });
    return () => { active = false; };
  }, [workspaceId, resourceId]);
  function acknowledge(saved: OriginalSource) {
    setList(value => value && ({ revision: saved.revision, sources: [...value.sources.filter(source => source.id !== saved.id), saved] }));
  }
  return { list, error, acknowledge };
}

function attachmentPending(state: AttachmentState) {
  return state.busy !== 'idle' || state.draft !== null || state.pending !== null;
}

function usePendingProtection(pending: boolean, onPendingChange?: (pending: boolean) => void) {
  const latest = useRef(onPendingChange);
  latest.current = onPendingChange;
  useEffect(() => { onPendingChange?.(pending); }, [onPendingChange, pending]);
  useEffect(() => () => { latest.current?.(false); }, []);
  useEffect(() => {
    if (!pending) return;
    function preventClose(event: BeforeUnloadEvent) { event.preventDefault(); event.returnValue = ''; }
    window.addEventListener('beforeunload', preventClose);
    return () => { window.removeEventListener('beforeunload', preventClose); };
  }, [pending]);
}

async function chooseSource(port: SourcePort, workspaceId: string, session: Session<AttachmentState>, requestFocus: () => void) {
  const current = session.generation.current;
  session.setState(value => ({ ...value, busy: 'picking', error: '', status: '' }));
  try {
    const selected = await port.pickSource({ contractVersion: 1, workspaceId, operationId: crypto.randomUUID() });
    if (session.generation.current === current && selected !== null) {
      session.setState(value => ({ ...value, draft: { attribution: '', license: '', originUrl: '', ...value.draft, selection: selected } }));
    }
  } catch (failure) {
    if (session.generation.current === current) session.setState(value => ({ ...value, error: failureMessage(failure) }));
  } finally {
    if (session.generation.current === current) {
      requestFocus();
      session.setState(value => ({ ...value, busy: 'idle' }));
    }
  }
}

function attachmentRequest(workspaceId: string, resourceId: string, revision: number, draft: Draft): SaveSource {
  return {
    contractVersion: 1, workspaceId, operationId: crypto.randomUUID(), expectedRevision: revision,
    source: { id: crypto.randomUUID(), resourceId, digest: draft.selection.digest, filename: draft.selection.filename,
      mediaType: 'application/octet-stream', attribution: draft.attribution || null, license: draft.license || null, originUrl: draft.originUrl || null },
  };
}

async function attachSource(port: SourcePort, workspaceId: string, resourceId: string, list: SourceList | null,
  session: Session<AttachmentState>, acknowledge: (saved: OriginalSource) => void) {
  const { draft, pending } = session.state;
  if (!draft || !list) return;
  const request = pending ?? attachmentRequest(workspaceId, resourceId, list.revision, draft);
  const current = session.generation.current;
  session.setState(value => ({ ...value, pending: request, busy: 'saving', error: '', status: '' }));
  try {
    const saved = await port.saveSource(request);
    if (session.generation.current !== current) return;
    acknowledge(saved);
    session.setState({ ...emptyAttachment, status: 'Attached original source.' });
  } catch (failure) {
    if (session.generation.current === current) session.setState(value => ({ ...value, busy: 'idle', error: failureMessage(failure) }));
  }
}

async function previewSource(port: SourcePort, sourceId: string, session: Session<PreviewState>) {
  const current = session.generation.current;
  session.setState({ ...emptyPreview, busy: true });
  try {
    const preview = await port.sourcePreview(sourceId);
    if (session.generation.current === current) session.setState({ preview, busy: false, error: '' });
  } catch (failure) {
    if (session.generation.current === current) session.setState({ ...emptyPreview, error: `Preview unavailable. ${failureMessage(failure)}` });
  }
}

function updateDraft(session: Session<AttachmentState>, field: 'attribution' | 'license' | 'originUrl', text: string) {
  session.setState(value => ({ ...value, draft: value.draft && { ...value.draft, [field]: text } }));
}

function SourceDraftForm({ draft, pending, busy, onChange, onAttach, onDiscard }: {
  draft: Draft; pending: boolean; busy: boolean;
  onChange: (field: 'attribution' | 'license' | 'originUrl', value: string) => void;
  onAttach: () => void; onDiscard: () => void;
}) {
  const locked = pending || busy;
  return <div>
    <p>Selected: {draft.selection.filename} ({draft.selection.byteLength} bytes)</p>
    <label>Attribution<input value={draft.attribution} disabled={locked} onChange={event => onChange('attribution', event.target.value)} /></label>
    <label>License<input value={draft.license} disabled={locked} onChange={event => onChange('license', event.target.value)} /></label>
    <label>Source URL<input value={draft.originUrl} disabled={locked} onChange={event => onChange('originUrl', event.target.value)} /></label>
    <button disabled={busy} onClick={onAttach}>{pending ? 'Retry same attachment' : 'Attach selected source'}</button>
    <button disabled={busy} onClick={onDiscard}>{pending ? 'Discard pending attachment' : 'Discard selected source'}</button>
  </div>;
}

function SourceRecord({ source, busy, onPreview }: { source: OriginalSource; busy: boolean; onPreview: (sourceId: string) => void }) {
  return <li>
    <strong>{source.filename}</strong> <span>({source.byteLength} bytes)</span>
    {source.attribution && <p>Attribution: <span>{source.attribution}</span></p>}
    {source.license && <p>License: <span>{source.license}</span></p>}
    {source.originUrl && <p>Source URL: <span>{source.originUrl}</span></p>}
    <button disabled={busy} onClick={() => onPreview(source.id)}>Preview original {source.filename}</button>
  </li>;
}

function OriginalPreview({ preview }: { preview: SourcePreview }) {
  return <section aria-label="Original preview">
    <h4>Original preview</h4>
    {preview.text !== null ? <pre>{preview.text}</pre> : <><p>Binary source: only a hexadecimal preview is available. Open the preserved original with a suitable application outside this preview.</p><pre>{preview.hex ?? 'No preview available.'}</pre></>}
    {preview.truncated && <p>Preview is truncated. The preserved original is unchanged.</p>}
  </section>;
}

function SourceMessages({ errors, status, loading }: { errors: string[]; status: string; loading: boolean }) {
  return <>
    {errors.filter(Boolean).map((error, index) => <p role="alert" key={index}>{error}</p>)}
    {status && <p role="status">{status}</p>}
    {loading && <p role="status">Loading sources…</p>}
  </>;
}

export function SourcesPanel({ workspaceId, resourceId, onPendingChange }: {
  workspaceId: string; resourceId: string; onPendingChange?: (pending: boolean) => void;
}) {
  const port = useSourcePort();
  const library = useSourceList(port, workspaceId, resourceId);
  const attachment = useSourceSession(workspaceId, resourceId, emptyAttachment);
  const preview = useSourceSession(workspaceId, resourceId, emptyPreview);
  const { state } = attachment;
  const busy = state.busy !== 'idle';
  const pickerFocus = usePickerFocus(state.busy === 'picking');
  usePendingProtection(attachmentPending(state), onPendingChange);
  return <section aria-label="Original sources">
    <h3>Original sources</h3>
    <p>Keep original files with their attribution and license. Originals are preserved separately from editable material.</p>
    <SourceMessages errors={[library.error, state.error, preview.state.error]} status={state.status} loading={!library.list && !library.error} />
    <button ref={pickerFocus.trigger} disabled={!library.list || busy || state.pending !== null}
      onClick={() => void chooseSource(port, workspaceId, attachment, pickerFocus.requestFocus)}>Choose source file</button>
    {state.draft && <SourceDraftForm draft={state.draft} pending={state.pending !== null} busy={busy}
      onChange={(field, value) => updateDraft(attachment, field, value)}
      onAttach={() => void attachSource(port, workspaceId, resourceId, library.list, attachment, library.acknowledge)}
      onDiscard={() => attachment.setState(emptyAttachment)} />}
    {library.list && <ul>{library.list.sources.map(source => <SourceRecord key={source.id} source={source} busy={preview.state.busy}
      onPreview={sourceId => void previewSource(port, sourceId, preview)} />)}</ul>}
    {preview.state.preview && <OriginalPreview preview={preview.state.preview} />}
  </section>;
}
