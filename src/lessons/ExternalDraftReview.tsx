import { useState } from 'react';
import { failureMessage } from '../application/errors';
import { LessonRenderer } from './LessonRenderer';
import type { DraftReview, DraftReviewResult, LessonPort } from './types';

type Summary = Extract<DraftReviewResult, { kind: 'list' }>['value'];
type Inspection = Extract<DraftReviewResult, { kind: 'inspection' }>['value'];

export function ExternalDraftReview({ unitId, review, onApplied }: {
  unitId: string;
  review: NonNullable<LessonPort['reviewExternalDraft']>;
  onApplied: () => void;
}) {
  const [drafts, setDrafts] = useState<Summary>();
  const [selected, setSelected] = useState<Inspection>();
  const [confirm, setConfirm] = useState<'apply' | 'reject'>();
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');

  async function run(action: DraftReview) {
    if (busy) return;
    setBusy(true);
    setError('');
    setConfirm(undefined);
    setMessage('');
    try {
      const result = await review(unitId, action);
      if (result.kind === 'inspection') {
        setSelected(result.value);
      } else if (result.kind === 'list') {
        setDrafts(result.value);
        setSelected(undefined);
      } else {
        setDrafts(previous => previous?.filter(draft => draft.id !== selected?.draft.id));
        setSelected(undefined);
        setMessage(result.kind === 'applied' ? 'Reviewed revision published.' : 'External draft rejected.');
        if (result.kind === 'applied') onApplied();
      }
    } catch (cause) {
      setError(failureMessage(cause));
    } finally {
      setBusy(false);
    }
  }

  return <details>
    <summary>External lesson drafts</summary>
    <button type="button" disabled={busy} onClick={() => void run({ kind: 'list' })}>Refresh external drafts</button>
    {drafts?.length === 0 && <p>No external drafts for this lesson.</p>}
    {drafts && drafts.length > 0 && <ul aria-label="External drafts">{drafts.map(draft => <li key={draft.id}>
      <button type="button" disabled={busy} onClick={() => void run({ kind: 'inspect', draftId: draft.id, revision: draft.revision })}>
        Inspect draft {draft.id}
      </button>
    </li>)}</ul>}
    {selected && <section aria-label="External draft preview">
      <h3>External draft preview</h3>
      <p>Revision {selected.draft.revision} · Externally authored · Unverified</p>
      {selected.validation.diagnostics.length > 0 && <ul aria-label="Validation issues">
        {selected.validation.diagnostics.map((issue, index) =>
          <li key={index}>{issue.field} · {issue.code}</li>)}
      </ul>}
      <LessonRenderer key={selected.draft.id + selected.draft.revision}
        composition={selected.draft.composition} {...selected.bindings} />
      {!confirm && <>
        {selected.reviewHash && <button type="button" disabled={busy} onClick={() => setConfirm('apply')}>Apply reviewed draft</button>}
        <button type="button" disabled={busy} onClick={() => setConfirm('reject')}>Reject external draft</button>
      </>}
      {confirm === 'apply' && selected.reviewHash && <div role="group" aria-label="Confirm reviewed revision">
        <p>Publish this exact reviewed revision to the lesson?</p>
        <button type="button" disabled={busy} onClick={() => void run({
          kind: 'apply', draftId: selected.draft.id, revision: selected.draft.revision,
          expectedHeadRevision: selected.draft.baseHeadRevision, hash: selected.reviewHash!,
          operationId: crypto.randomUUID(),
        })}>Confirm publication</button>
        <button type="button" onClick={() => setConfirm(undefined)}>Keep draft</button>
      </div>}
      {confirm === 'reject' && <div role="group" aria-label="Confirm draft rejection">
        <p>Reject this unpublished draft and remove it from the course?</p>
        <button type="button" disabled={busy} onClick={() => void run({
          kind: 'reject', draftId: selected.draft.id, revision: selected.draft.revision, operationId: crypto.randomUUID(),
        })}>Confirm rejection</button>
        <button type="button" onClick={() => setConfirm(undefined)}>Keep draft</button>
      </div>}
    </section>}
    {busy && <p role="status">Loading external drafts…</p>}
    {error && <p role="alert">{error}</p>}
    {message && <p role="status">{message}</p>}
  </details>;
}
