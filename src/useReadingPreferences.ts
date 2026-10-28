import { useEffect, useState } from 'react';
import { reopenWorkspace, saveReadingPreferences, workspaceStatus } from './workspaceBridge';
import type { PreferencesSnapshot, ReadingPreferences, SavePreferences, WorkspaceStatus } from './workspaceBridge';

const errors: Record<string, string> = {
  already_open: 'This workspace is open in another Memoquill process. Close it there, then retry.',
  busy: 'The workspace is busy. Your changes are still here; retry when it is available.',
  unsupported_schema: 'This workspace was created by an unsupported version. Use a compatible Memoquill version; the workspace was not replaced.',
  revision_conflict: 'Saved preferences changed. Reload saved preferences before making a new change.',
  idempotency_conflict: 'This operation conflicts with an earlier request. Reload saved preferences before continuing.',
  storage_full: 'Storage is full. Free space, then retry. Your material edits are still here.',
  recovery_required: 'The workspace needs recovery. Sources and records were preserved; saving is unavailable.',
  outcome_unconfirmed: 'The save could not be confirmed. Retry the same operation to check its outcome safely.',
  owner_unavailable: 'The storage owner is unavailable. Retry to reopen the workspace and check the operation.',
  permission_denied: 'This operation is not permitted. Your material edits are still here.',
};
export function storageMessage(reason: unknown): string {
  const code = typeof reason === 'object' && reason !== null && 'code' in reason ? reason.code : null;
  return typeof code === 'string' && Object.hasOwn(errors, code)
    ? errors[code] : 'The workspace action failed. Your material edits are still here; retry or inspect recovery options.';
}
function needsRecovery(status: WorkspaceStatus): boolean {
  return status.recovery.missingAssets + status.recovery.corruptAssets + status.recovery.unsafePaths > 0;
}
function readyMessage(status: WorkspaceStatus): string {
  if (needsRecovery(status)) return errors.recovery_required;
  const retained = status.recovery.stagingFiles + status.recovery.orphanAssets;
  return retained > 0
    ? `Workspace ready. ${retained} unreferenced files were preserved for later recovery. Only reading preferences are saved.`
    : 'Workspace ready. Reading preferences are saved locally; material edits are temporary.';
}
async function sendPreferences(request: SavePreferences, retry: boolean): Promise<PreferencesSnapshot> {
  if (retry) await reopenWorkspace();
  return saveReadingPreferences(request);
}
function useInitialWorkspace(accept: (status: WorkspaceStatus) => void,
  fail: (reason: unknown) => void, finish: () => void) {
  useEffect(() => {
    let active = true;
    workspaceStatus().then(status => { if (active) accept(status); })
      .catch(reason => { if (active) fail(reason); })
      .finally(() => { if (active) finish(); });
    return () => { active = false; };
  }, []); // Startup only: callbacks use stable state setters, never current edits.
}
export function useReadingPreferences() {
  const [snapshot, setSnapshot] = useState<PreferencesSnapshot | null>(null);
  const [preferences, setPreferences] = useState<ReadingPreferences>({ theme: 'system', textScale: 100 });
  const [pending, setPending] = useState<SavePreferences | null>(null);
  const [working, setWorking] = useState(true);
  const [recovery, setRecovery] = useState(false);
  const [message, setMessage] = useState('Opening local workspace…');

  function accept(status: WorkspaceStatus) {
    if (status.contractVersion !== 1) throw new Error('Unsupported workspace contract');
    setSnapshot(status.preferences);setPreferences(status.preferences.preferences);
    setRecovery(needsRecovery(status));setPending(null);setMessage(readyMessage(status));
  }
  useInitialWorkspace(accept, reason => setMessage(storageMessage(reason)), () => setWorking(false));
  useEffect(() => { document.documentElement.style.fontSize = `${preferences.textScale}%`; }, [preferences.textScale]);

  async function reload(): Promise<string> {
    setWorking(true);
    try { const status = await reopenWorkspace();accept(status);return readyMessage(status); }
    catch (reason) { const text = storageMessage(reason);setMessage(text);return text; }
    finally { setWorking(false); }
  }
  async function save(): Promise<string> {
    if (!snapshot) return 'Open the workspace before saving preferences.';
    const request = pending ?? { contractVersion: 1, workspaceId: snapshot.workspaceId,
      operationId: crypto.randomUUID(), expectedRevision: snapshot.revision, preferences } satisfies SavePreferences;
    setPending(request);setWorking(true);
    try {
      const result = await sendPreferences(request, pending !== null);
      setSnapshot(result);setPreferences(result.preferences);setPending(null);
      const text = 'Reading preferences saved. Material edits are still temporary.';
      setMessage(text);return text;
    } catch (reason) { const text = storageMessage(reason);setMessage(text);return text; }
    finally { setWorking(false); }
  }
  return { preferences, setPreferences, working, recovery, message, save, reload,
    loaded: snapshot !== null, retry: pending !== null };
}
export type ReadingController = ReturnType<typeof useReadingPreferences>;
