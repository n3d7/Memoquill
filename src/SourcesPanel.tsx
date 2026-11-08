import type { OriginalSource, SourcePreview } from './sourceBridge';
import { useSourceController, type SourceController } from './application/useSourceController';

function SourceDraftForm({ draft, pending, busy, onChange, onAttach, onDiscard }: {
  draft: NonNullable<SourceController['state']['draft']>; pending: boolean; busy: boolean;
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

/** Standalone compatibility entry point; MaterialPanel owns this controller itself. */
export function SourcesPanel({ workspaceId, resourceId, onPendingChange }: {
  workspaceId: string; resourceId: string; onPendingChange?: (pending: boolean) => void;
}) {
  const model = useSourceController(workspaceId, resourceId, onPendingChange);
  return <SourcesView model={model} />;
}
export function SourcesView({ model }: { model: SourceController }) {
  const { state, busy } = model;
  return <section aria-label="Original sources">
    <h3>Original sources</h3>
    <p>Keep original files with their attribution and license. Originals are preserved separately from editable material.</p>
    <SourceMessages errors={[model.error, state.error, model.preview.error]} status={state.status} loading={!model.list && !model.error} />
    <button ref={model.pickerTrigger} disabled={!model.list || busy || state.pending !== null}
      onClick={() => void model.choose()}>Choose source file</button>
    {state.draft && <SourceDraftForm draft={state.draft} pending={state.pending !== null} busy={busy}
      onChange={(field, value) => model.updateDraft(field, value)}
      onAttach={() => void model.attach()}
      onDiscard={() => model.discard()} />}
    {model.list && <ul>{model.list.sources.map(source => <SourceRecord key={source.id} source={source} busy={model.preview.busy}
      onPreview={sourceId => void model.previewSource(sourceId)} />)}</ul>}
    {model.preview.preview && <OriginalPreview preview={model.preview.preview} />}
  </section>;
}
