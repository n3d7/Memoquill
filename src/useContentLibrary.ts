import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection, type ApplyContent, type Command, type Library } from './contentBridge';

import { useContentPort } from './application/ApplicationProvider';

export function useContentLibrary() {
  const { library, applyContent } = useContentPort();
  const [snapshot, setSnapshot] = useState<Library | null>(null);
  const current = useRef<Library | null>(null);
  const pending = useRef<ApplyContent | null>(null);
  const active = useRef(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('Opening local workspace…');
  const [error, setError] = useState(false);
  function accept(value: Library) { current.current = value; setSnapshot(value); }
  async function reload() {
    if (active.current) return;
    active.current = true; setBusy(true);
    try { accept(await library()); setMessage(pending.current ? 'Workspace refreshed. Retry the unresolved change before making another change.' : 'Local workspace ready.'); setError(pending.current !== null); }
    catch (reason) { setMessage(failureMessage(reason)); setError(true); }
    finally { active.current = false; setBusy(false); }
  }
  useEffect(() => { let live = true; void library().then(value => { if (live) { accept(value); setMessage('Local workspace ready.'); } }, reason => { if (live) { setMessage(failureMessage(reason)); setError(true); } }); return () => { live = false; }; }, []);
  async function send(request: ApplyContent): Promise<Library | null> {
    if (active.current) return null;
    active.current = true; setBusy(true); setError(false); pending.current = request;
    try { const value = await applyContent(request); accept(value); pending.current = null; setMessage('Change saved locally.'); return value; }
    catch (reason) { if (isConfirmedRejection(reason)) pending.current = null; setMessage(failureMessage(reason)); setError(true); return null; }
    finally { active.current = false; setBusy(false); }
  }
  async function perform(command: Command) {
    if (!current.current || pending.current) return null;
    return send({ contractVersion: 1, workspaceId: current.current.workspaceId, operationId: crypto.randomUUID(), expectedRevision: current.current.revision, command });
  }
  function retry() { return pending.current ? send(pending.current) : reload(); }
  return { snapshot, busy, message, error, hasPending: pending.current !== null, perform, retry, reload };
}
