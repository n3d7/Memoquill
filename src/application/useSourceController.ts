import { useEffect, useRef, useState } from 'react';
import type { Dispatch, RefObject, SetStateAction } from 'react';
import { failureMessage } from '../contentBridge';
import { useSourcePort } from './ApplicationProvider';
import type { SourcePort } from './ports';
import type { OriginalSource, SaveSource, SelectedSource, SourceList, SourcePreview } from '../sourceBridge';
import { usePickerFocus } from '../usePickerFocus';

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

/** Source scenario ownership remains mounted independently of its presentation. */
export function useSourceController(workspaceId: string, resourceId: string, onPendingChange?: (pending: boolean) => void) {
  const port = useSourcePort();
  const library = useSourceList(port, workspaceId, resourceId);
  const attachment = useSourceSession(workspaceId, resourceId, emptyAttachment);
  const preview = useSourceSession(workspaceId, resourceId, emptyPreview);
  const { state } = attachment;
  const busy = state.busy !== 'idle';
  const pickerFocus = usePickerFocus(state.busy === 'picking');
  const pending = attachmentPending(state);
  usePendingProtection(pending, onPendingChange);
  return {
    state, busy, pending, list: library.list, error: library.error, preview: preview.state,
    pickerTrigger: pickerFocus.trigger,
    choose: () => chooseSource(port, workspaceId, attachment, pickerFocus.requestFocus),
    attach: () => attachSource(port, workspaceId, resourceId, library.list, attachment, library.acknowledge),
    discard: () => attachment.setState(emptyAttachment),
    updateDraft: (field: 'attribution' | 'license' | 'originUrl', value: string) => updateDraft(attachment, field, value),
    previewSource: (sourceId: string) => previewSource(port, sourceId, preview),
  };
}
export type SourceController = ReturnType<typeof useSourceController>;
