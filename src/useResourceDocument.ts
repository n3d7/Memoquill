import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection, readDocument, saveDocument, type SaveDocument, type SavedDocument } from './contentBridge';
import { emptyDocument, type Document } from './documentModel';

export function useResourceDocument(workspaceId: string, resourceId: string) {
  const [document, setDocument] = useState<Document>(emptyDocument);
  const current = useRef(document);
  const [saved, setSaved] = useState<SavedDocument | null>(null);
  const savedRef = useRef(saved);
  const pending = useRef<SaveDocument | null>(null);
  const saving = useRef(false);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [dirty, setDirty] = useState(false);
  const [message, setMessage] = useState('Opening material…');
  const [error, setError] = useState(false);
  function change(value: Document) { current.current = value; setDocument(value); setDirty(true); }
  useEffect(() => {
    let live = true;
    void readDocument(resourceId).then(value => {
      if (!live) return;
      const doc = value?.document ?? emptyDocument(); current.current = doc; setDocument(doc); savedRef.current = value; setSaved(value); setLoading(false); setDirty(false); setMessage(value ? 'Saved material opened.' : 'New material. Nothing saved yet.');
    }, reason => { if (live) { setError(true); setMessage(failureMessage(reason)); setLoading(false); } });
    return () => { live = false; };
  }, [resourceId]);
  useEffect(() => { function warn(event: BeforeUnloadEvent) { if (dirty) { event.preventDefault(); } } window.addEventListener('beforeunload', warn); return () => window.removeEventListener('beforeunload', warn); }, [dirty]);
  async function send(request: SaveDocument) {
    if (saving.current) return;
    saving.current = true; setBusy(true); setError(false); pending.current = request; setMessage('Saving locally…');
    try {
      const value = await saveDocument(request); savedRef.current = value; setSaved(value); pending.current = null;
      const unchanged = current.current === request.document; setDirty(!unchanged);
      setMessage(unchanged ? 'Saved locally.' : 'Earlier draft saved. Newer edits still need saving.');
    } catch (reason) { if (isConfirmedRejection(reason)) pending.current = null; setError(true); setMessage(failureMessage(reason)); }
    finally { saving.current = false; setBusy(false); }
  }
  function save() {
    if (pending.current) return;
    return send({ contractVersion: 1, workspaceId, operationId: crypto.randomUUID(), expectedRevision: savedRef.current?.revision ?? 0, resourceId, document: current.current });
  }
  function retry() { return pending.current ? send(pending.current) : save(); }
  function discard() { if (pending.current || saving.current) return; const value = savedRef.current?.document ?? emptyDocument(); current.current = value; setDocument(value); setDirty(false); setError(false); setMessage('Unsaved changes discarded.'); }
  async function reloadSaved() {
    if (pending.current || saving.current) return;
    saving.current = true; setBusy(true);
    try { const value = await readDocument(resourceId); savedRef.current = value; setSaved(value); setError(false); setMessage('Latest saved revision loaded. Your draft is retained; review it before saving.'); }
    catch (reason) { setError(true); setMessage(failureMessage(reason)); }
    finally { saving.current = false; setBusy(false); }
  }
  return { document, saved, dirty, busy, loading, message, error, hasPending: pending.current !== null, change, save, retry, discard, reloadSaved };
}
