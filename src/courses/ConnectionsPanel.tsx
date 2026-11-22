import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { failureMessage } from '../application/errors';

interface Permissions { content: boolean; progress: boolean; responses: boolean; sources: boolean; originals: boolean; drafts: boolean; delegateAdditions: boolean; delegateDocumentEdits: boolean }
interface Peer {
  challenge: string;
  pid: number | null;
  courseId: string | null;
  permissions: Permissions | null;
}
interface AuditEntry {
  action: 'enabled' | 'disabled' | 'approved' | 'revoked' | 'disconnected' | 'expired';
  atUnixMs: number;
  courseId: string | null;
}
interface Status { enabled: boolean; peers: Peer[]; recentEvents?: AuditEntry[] }

export interface ConnectionsPort {
  status(): Promise<Status>;
  setEnabled(enabled: boolean): Promise<Status>;
  approve(challenge: string, courseId: string, permissions: Permissions): Promise<Status>;
  revoke(challenge: string): Promise<Status>;
}

export const nativeConnectionsPort: ConnectionsPort = {
  status: () => invoke('mcp_status'),
  setEnabled: enabled => invoke('mcp_set_enabled', { enabled }),
  approve: (challenge, courseId, permissions) => invoke('mcp_approve', { challenge, courseId, permissions }),
  revoke: challenge => invoke('mcp_revoke', { challenge }),
};

export function ConnectionsPanel({ course, port }: {
  course: { id: string; title: string };
  port: ConnectionsPort;
}) {
  const [status, setStatus] = useState<Status | null>(null);
  const [challenge, setChallenge] = useState('');
  const [content, setContent] = useState(false);
  const [progress, setProgress] = useState(false);
  const [sources, setSources] = useState(false);
  const [responses, setResponses] = useState(false);
  const [originals, setOriginals] = useState(false);
  const [drafts, setDrafts] = useState(false);
  const [delegateAdditions, setDelegateAdditions] = useState(false);
  const [delegateDocumentEdits, setDelegateDocumentEdits] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const pending = status?.peers.find(peer => peer.courseId === null && peer.challenge === challenge.trim());

  async function update(action: () => Promise<Status>) {
    setBusy(true);
    setError('');
    try {
      setStatus(await action());
    } catch (cause) {
      setError(failureMessage(cause));
    } finally {
      setBusy(false);
    }
  }

  return <details>
    <summary onClick={() => { if (!status && !busy) void update(port.status); }}>Connections</summary>
    <p>Pair a local MCP host with the selected course. An approved host can copy or transmit the material it reads.</p>
    <p>Selected course: <strong>{course.title}</strong></p>
    <button type="button" disabled={busy} onClick={() => void update(port.status)}>Refresh connections</button>
    {status && <button type="button" disabled={busy}
      onClick={() => void update(() => port.setEnabled(!status.enabled))}>
      {status.enabled ? 'Disable connections and revoke all grants' : 'Enable local connections'}
    </button>}
    {status?.enabled && <>
      <p>Call <code>memoquill_connection</code> in the host, enter its challenge here, then refresh connections.</p>
      <form onSubmit={event => {
        event.preventDefault();
        if (!pending || (!content && !progress)) return;
        void update(async () => {
          const updated = await port.approve(pending.challenge, course.id, {
            content, progress, responses: progress && responses,
            sources: content && sources, originals: content && sources && originals, drafts: content && drafts, delegateAdditions: content && drafts && delegateAdditions,
            delegateDocumentEdits: content && drafts && delegateAdditions && delegateDocumentEdits,
          });
          setChallenge('');
          setContent(false);
          setProgress(false);
          setSources(false);
          setResponses(false);
          setOriginals(false);
          setDrafts(false);
          setDelegateAdditions(false);
          setDelegateDocumentEdits(false);
          return updated;
        });
      }}>
        <fieldset disabled={busy}>
          <legend>Approve a connection</legend>
          <label>Pairing challenge<input value={challenge} maxLength={40}
            spellCheck={false} autoComplete="off" onChange={event => setChallenge(event.target.value)} /></label>
          {pending && <p>Bridge process {pending.pid ?? 'unknown'}</p>}
          <label><input type="checkbox" checked={content} onChange={event => setContent(event.target.checked)} />
            Published course structure and normalized document text</label>
          <label><input type="checkbox" checked={progress} onChange={event => setProgress(event.target.checked)} />
            Aggregate lesson progress</label>
          <label><input type="checkbox" disabled={!progress} checked={progress && responses}
            onChange={event => setResponses(event.target.checked)} />Raw learner responses</label>
          <label><input type="checkbox" disabled={!content} checked={content && sources}
            onChange={event => setSources(event.target.checked)} />Extracted records from published lesson sources</label>
          <label><input type="checkbox" disabled={!content || !sources} checked={content && sources && originals}
            onChange={event => setOriginals(event.target.checked)} />Original bytes from published lesson sources</label>
          <label><input type="checkbox" disabled={!content} checked={content && drafts}
            onChange={event => setDrafts(event.target.checked)} />Create and edit unpublished course and lesson drafts</label>
          <label><input type="checkbox" disabled={!content || !drafts} checked={content && drafts && delegateAdditions}
            onChange={event => setDelegateAdditions(event.target.checked)} />
            Delegate safe course additions (32 operations)</label>
          <label><input type="checkbox" disabled={!content || !drafts || !delegateAdditions}
            checked={content && drafts && delegateAdditions && delegateDocumentEdits}
            onChange={event => setDelegateDocumentEdits(event.target.checked)} />
            Delegate edits to the agent's unlinked documents</label>
          <button type="submit" disabled={!pending || (!content && !progress)}>Approve for {course.title}</button>
        </fieldset>
      </form>
      <ul aria-label="Approved connections">
        {status.peers.filter(peer => peer.courseId !== null).map(peer => <li key={peer.challenge}>
          Bridge process {peer.pid ?? 'unknown'} · {peer.courseId === course.id ? course.title : peer.courseId}
          {peer.permissions?.content && ' · Course content'}
          {peer.permissions?.progress && ' · Aggregate progress'}
          {peer.permissions?.sources && ' · Lesson source records'}
          {peer.permissions?.originals && ' · Original bytes'}
          {peer.permissions?.responses && ' · Raw responses'}
          {peer.permissions?.drafts && ' · Unpublished drafts'}
          {peer.permissions?.delegateAdditions && ' · Delegated additions'}
          {peer.permissions?.delegateDocumentEdits && ' · Delegated document edits'}
          {' '}<button type="button" aria-label={`Revoke bridge ${peer.pid ?? 'unknown'}`} disabled={busy} onClick={() => void update(() => port.revoke(peer.challenge))}>
            Revoke
          </button>
        </li>)}
      </ul>
    </>}
    {status?.recentEvents && status.recentEvents.length > 0 && <details>
      <summary>Recent connection events</summary>
      <ul>{[...status.recentEvents].reverse().map((event, index) => <li key={index}>
        {new Date(event.atUnixMs).toLocaleTimeString()} · {event.action}
        {event.courseId && ` · ${event.courseId === course.id ? course.title : event.courseId}`}
      </li>)}</ul>
    </details>}
    {busy && <p role="status">Updating connections…</p>}
    {error && <p role="alert">{error}</p>}
  </details>;
}
