import { type Command, type Library } from './contentBridge';
import { DocumentEditor } from './DocumentEditor';
import { useMaterialController } from './application/useMaterialController';
import { NameForm } from './OrganizationForms';
import { SourcesView } from './SourcesPanel';
import type { SavedContextModel } from './application/useSavedContext';

import { ActionButton, StatusMessage } from './application/controls';
import type { ComponentType } from 'react';
import { useOptionalIngestionPort } from './application/ApplicationProvider';
import { IngestionPanel } from './sources/IngestionPanel';

type Perform = (command: Command) => Promise<Library | null>;
export interface MaterialPanelProps { data: Library; resourceId: string; onDirty: (dirty: boolean) => void; perform: Perform; onLibraryChanged?: () => void; view?: ComponentType<MaterialViewProps> }
export type MaterialViewProps = MaterialPanelProps & ReturnType<typeof useMaterialController>;
export function MaterialPanel(props: MaterialPanelProps) {
  const controller = useMaterialController(props.data.workspaceId, props.resourceId, props.onDirty);
  const View = props.view ?? MaterialView;
  return <View {...props} {...controller} />;
}
export function MaterialView({ data, resourceId, perform, material, resume, contextPending, context, sources, setIngestionPending, onLibraryChanged }: MaterialViewProps) {
  const ingestion = useOptionalIngestionPort();
  return <section aria-label="Material authoring"><h2>{data.resources[resourceId].title}</h2>
    <p>Shared material: changes apply to every placement. Content revisions and original sources are retained.</p>
    <NameForm label="New material title" action="Rename material" onSubmit={title => perform({ type: 'renameResource', id: resourceId, title })} />
    <NameForm label="Tag for this material" action="Add tag" onSubmit={title => { const existing = Object.values(data.tags).find(x => x.title === title); return perform({ type: 'setTag', id: existing?.id ?? crypto.randomUUID(), title, resourceIds: [...new Set([...(existing?.resourceIds ?? []), resourceId])] }); }} />
    <StatusMessage>{material.dirty ? 'Unsaved changes. ' : ''}{material.message}</StatusMessage>
    {material.error && <p role="alert">{material.message}</p>}
    {!material.loading && <DocumentEditor key={resourceId} document={material.document} onChange={material.change} resume={resume} />}
    {contextPending && <p>Finish or clear the annotation draft and resolve pending reading changes before saving a new material revision.</p>}
    <div className="actions"><ActionButton disabled={material.loading || material.busy || material.hasPending || contextPending} onClick={() => void material.save()}>Save material</ActionButton>
      <ActionButton disabled={material.busy || material.hasPending || !material.dirty} onClick={material.discard}>Discard unsaved changes</ActionButton>
      {material.error && <ActionButton disabled={material.busy} onClick={() => void material.retry()}>Retry same save</ActionButton>}
      {material.error && !material.hasPending && <ActionButton disabled={material.busy || contextPending} onClick={() => void material.reloadSaved()}>Reload saved revision; keep draft</ActionButton>}
    </div>
    {context && <SavedContextView model={context} />}
    <SourcesView model={sources} />
  {ingestion && sources.list && <IngestionPanel key={`${data.workspaceId}:${resourceId}`} port={ingestion} sources={sources.list.sources} workspaceId={data.workspaceId} resourceId={resourceId} libraryRevision={data.revision} onChanged={onLibraryChanged ?? (() => {})} onPendingChange={setIngestionPending} />}
  </section>;
}
function SavedContextView({ model }: { model: SavedContextModel }) {
  const { saved, blockId, setBlockId, notes, body, setBody, message, busy, loading, locked, hasPending, remember, addNote, loadContext, retry, resume } = model;
  if (!saved) return null;
  return <fieldset disabled={busy || loading || locked}><legend>Saved revision {saved.revision}</legend>
    <label>Reading and annotation block<select value={blockId} onChange={event => setBlockId(event.target.value)}>{saved.document.blocks.map((block, index) => <option key={block.id} value={block.id}>Block {index + 1}: {block.content.type}</option>)}</select></label>
    <button disabled={hasPending} onClick={() => void remember()}>Remember reading location</button>
    <button onClick={() => resume()}>Resume reading here</button>
    <p role="status">{message}</p>
    <button disabled={hasPending} onClick={() => void loadContext()}>Reload saved context</button>
    {hasPending && <button onClick={() => void retry()}>Retry saved context</button>}
    <form onSubmit={event => { event.preventDefault(); void addNote(); }}><label>Annotation at start of block<textarea value={body} required maxLength={16384} onChange={event => setBody(event.target.value)} /></label><button disabled={hasPending}>Add annotation</button></form>
    <ul>{notes.map(note => <li key={note.id}>{note.body} {note.revisionId !== saved.id && <strong>— attached to an older revision; reattachment is unverified</strong>}</li>)}</ul>
  </fieldset>;
}
