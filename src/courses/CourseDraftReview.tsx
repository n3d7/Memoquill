import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { failureMessage } from '../application/errors';

type Summary = {
  id: string;
  title: string;
  baseRevision: number;
  revision: number;
  status: 'submitted' | 'applied' | 'undone';
  operations: number;
};
type Inspection = {
  draft: Summary;
  hash: string;
  operations: unknown[];
  nextOffset: number | null;
  undoAvailable: boolean;
  validation: { valid: boolean; operationIndex?: number; code?: string };
};
type Request =
  | { kind: 'list' }
  | { kind: 'inspect'; draftId: string; revision: number; offset: number }
  | { kind: 'undo'; draftId: string; revision: number; operationId: string }
  | { kind: 'apply'; draftId: string; revision: number; hash: string; operationId: string };

export function CourseDraftReview({ courseId, onChanged }: {
  courseId: string;
  onChanged: () => Promise<void>;
}) {
  const [drafts, setDrafts] = useState<Summary[]>();
  const [inspection, setInspection] = useState<Inspection>();
  const [offset, setOffset] = useState(0);
  const [confirm, setConfirm] = useState<'apply' | 'undo'>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');

  async function command<T>(request: Request, accept: (value: T) => void | Promise<void>) {
    if (busy) return;
    setBusy(true);
    setError('');
    setMessage('');
    setConfirm(undefined);
    try {
      await accept(await invoke<T>('review_course_draft', { courseId, request }));
    } catch (cause) {
      setError(failureMessage(cause));
    } finally {
      setBusy(false);
    }
  }

  function refresh() {
    return command<{ drafts: Summary[] }>({ kind: 'list' }, result => {
      setDrafts(result.drafts);
      setInspection(undefined);
    });
  }

  function inspect(draft: Summary, offset: number) {
    return command<Inspection>({ kind: 'inspect', draftId: draft.id, revision: draft.revision, offset }, result => {
      setInspection(result);
      setOffset(offset);
    });
  }

  async function apply() {
    if (!inspection) return;
    await command<Summary>({
      kind: 'apply', draftId: inspection.draft.id, revision: inspection.draft.revision,
      hash: inspection.hash, operationId: crypto.randomUUID(),
    }, async result => {
      setDrafts(previous => previous?.map(draft => draft.id === result.id ? result : draft));
      setInspection(undefined);
      setMessage('The exact reviewed group was applied.');
      await onChanged();
    });
  }

  async function undo() {
    if (!inspection) return;
    await command<Summary>({
      kind: 'undo', draftId: inspection.draft.id, revision: inspection.draft.revision,
      operationId: crypto.randomUUID(),
    }, async result => {
      setDrafts(previous => previous?.map(draft => draft.id === result.id ? result : draft));
      setInspection(undefined);
      setMessage('The additions were archived.');
      await onChanged();
    });
  }

  return <details>
    <summary>Course draft proposals</summary>
    <button type="button" disabled={busy} onClick={() => void refresh()}>Refresh course proposals</button>
    {drafts?.length === 0 && <p>No submitted course proposals.</p>}
    {!!drafts?.length && <ul aria-label="Course proposals">{drafts.map(draft => <li key={draft.id}>
      <button type="button" disabled={busy} onClick={() => void inspect(draft, 0)}>
        Inspect {draft.title}
      </button> {draft.status}
    </li>)}</ul>}
    {inspection && <section aria-label="Course proposal preview">
      <h3>{inspection.draft.title}</h3>
      <p>{inspection.draft.operations} operations · Externally authored · Unverified</p>
      {!inspection.validation.valid && inspection.draft.status === 'submitted' &&
        <p role="alert">Proposal needs revision. {inspection.validation.code}</p>}
      <ol start={offset + 1}>{inspection.operations.map((operation, index) =>
        <li key={offset + index}><pre>{JSON.stringify(operation, null, 2)}</pre></li>)}</ol>
      {inspection.nextOffset !== null && <button type="button" disabled={busy}
        onClick={() => void inspect(inspection.draft, inspection.nextOffset!)}>Review next operations</button>}
      {inspection.nextOffset === null && inspection.draft.status === 'submitted' && inspection.validation.valid && <>
        {!confirm && <button type="button" disabled={busy} onClick={() => setConfirm('apply')}>Apply reviewed course group</button>}
        {confirm === 'apply' && <div role="group" aria-label="Confirm reviewed course group">
          <p>Apply all reviewed operations in this group to the selected course?</p>
          <button type="button" disabled={busy} onClick={() => void apply()}>Confirm course group</button>
          <button type="button" onClick={() => setConfirm(undefined)}>Keep proposal</button>
        </div>}
      </>}
      {inspection.undoAvailable && <>
        {confirm !== 'undo' && <button type="button" disabled={busy} onClick={() => setConfirm('undo')}>Undo group additions</button>}
        {confirm === 'undo' && <div role="group" aria-label="Confirm undo">
          <p>Archive the additions made by this group?</p>
          <button type="button" disabled={busy} onClick={() => void undo()}>Confirm undo</button>
          <button type="button" onClick={() => setConfirm(undefined)}>Keep additions</button>
        </div>}
      </>}
    </section>}
    {busy && <p role="status">Loading course proposals…</p>}
    {error && <p role="alert">{error}</p>}
    {message && <p role="status">{message}</p>}
  </details>;
}
