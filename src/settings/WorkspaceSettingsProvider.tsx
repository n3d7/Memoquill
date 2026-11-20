import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import { storageMessage } from '../useReadingPreferences';
import { emptyLayer, type SettingsLayer, type SettingsPort, type SaveWorkspaceSettings, type WorkspaceSettings } from './types';
function useWorkspaceSettings(port?: SettingsPort) {
  const [snapshot, setSnapshot] = useState<WorkspaceSettings | null>(null);
  const [pending, setPending] = useState<SaveWorkspaceSettings | null>(null);
  const [busy, setBusy] = useState(false); const [message, setMessage] = useState('');
  const lock = useRef(false);
  async function reload() {
    if (!port || lock.current) return;
    lock.current = true; setBusy(true);
    try { setSnapshot(await port.read()); setPending(null); setMessage('Workspace defaults loaded.'); }
    catch (reason) { setMessage(storageMessage(reason)); }
    finally { lock.current = false; setBusy(false); }
  }
  useEffect(() => { void reload(); }, [port]);
  async function save(settings: SettingsLayer) {
    if (!port || !snapshot || lock.current) return;
    const request = pending ?? { operationId: crypto.randomUUID(), expectedRevision: snapshot.revision, settings };
    lock.current = true; setBusy(true); setPending(request);
    try { setSnapshot(await port.save(request)); setPending(null); setMessage('Workspace defaults saved. Course and session overrides are retained.'); return true; }
    catch (reason) { setMessage(storageMessage(reason)); return false; }
    finally { lock.current = false; setBusy(false); }
  }
  return { settings: snapshot?.settings ?? emptyLayer(), loaded: snapshot !== null, available: !!port, busy, pending: pending !== null, pendingSettings: pending?.settings, message, reload, save };
}
type Controller = ReturnType<typeof useWorkspaceSettings>;
const Context = createContext<Controller | null>(null);
export const useWorkspaceDefaults = () => useContext(Context);
export function WorkspaceSettingsProvider({ port, children }: { port?: SettingsPort; children: ReactNode }) {
  const controller = useWorkspaceSettings(port);
  return <Context.Provider value={controller}>{children}</Context.Provider>;
}
