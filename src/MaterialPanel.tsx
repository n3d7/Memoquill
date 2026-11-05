import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection, type Annotation, type Command, type Library, type ReadingPosition, type SavedDocument } from './contentBridge';
import { DocumentEditor } from './DocumentEditor';
import { useMaterialController } from './application/useMaterialController';
import { NameForm } from './OrganizationForms';
import { SourcesPanel } from './SourcesPanel';

import { useContentPort } from './application/ApplicationProvider';
import type { ContentPort } from './application/ports';
import { ActionButton, StatusMessage } from './application/controls';
import type { ComponentType } from 'react';

type Perform = (command: Command) => Promise<Library | null>;
export interface MaterialPanelProps { data: Library; resourceId: string; onDirty: (dirty: boolean) => void; perform: Perform; view?: ComponentType<MaterialViewProps> }
export type MaterialViewProps = MaterialPanelProps & ReturnType<typeof useMaterialController>;
export function MaterialPanel(props: MaterialPanelProps) {
  const controller = useMaterialController(props.data.workspaceId, props.resourceId, props.onDirty);
  const View = props.view ?? MaterialView;
  return <View {...props} {...controller} />;
}
export function MaterialView({ data, resourceId, perform, material, resume, setResume, contextPending, setContextPending, setSourcePending }: MaterialViewProps) {
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
    {material.saved && <SavedContext key={material.saved.id} saved={material.saved} workspaceId={data.workspaceId} onResume={setResume} onDirty={setContextPending} locked={material.busy || material.hasPending} />}
    <SourcesPanel key={`sources:${resourceId}`} workspaceId={data.workspaceId} resourceId={resourceId} onPendingChange={setSourcePending} />
  </section>;
}
type ContextRequest = { kind: 'position'; request: Parameters<ContentPort['saveReadingPosition']>[0] } | { kind: 'annotation'; request: Parameters<ContentPort['saveAnnotation']>[0] };
function SavedContext({ saved, workspaceId, onResume, onDirty, locked }: { saved: SavedDocument; workspaceId: string; onResume: (value: { blockId: string; offsetUtf16: number }) => void; onDirty: (value: boolean) => void; locked: boolean }) {
  const { readingPosition, annotations, saveReadingPosition, saveAnnotation } = useContentPort();
  const [blockId, setBlockId] = useState(saved.document.blocks[0]?.id ?? '');
  const [position, setPosition] = useState<ReadingPosition | null>(null);
  const [notes, setNotes] = useState<Annotation[]>([]); const [body, setBody] = useState('');
  const [message, setMessage] = useState(''); const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const pending = useRef<ContextRequest | null>(null); const active = useRef(false);
  const loadGeneration = useRef(0);
  const pendingRequest = pending.current;
  useEffect(() => { onDirty(body.length > 0 || pendingRequest !== null); return () => onDirty(false); }, [body, pendingRequest, onDirty]);
  async function loadContext(live: () => boolean = () => true) {
    if (active.current || pending.current) return;
    const generation = ++loadGeneration.current; setLoading(true);
    try {
      const [place, list] = await Promise.all([readingPosition(saved.resourceId), annotations(saved.resourceId)]);
      if (!live() || generation !== loadGeneration.current) return; setPosition(place); setNotes(list);
      if (place?.revisionId === saved.id && saved.document.blocks.some(x => x.id === place.blockId)) {
        setBlockId(place.blockId); setMessage('Saved reading location restored.');
      } else if (place) setMessage('The saved location belongs to an older revision. Choose a block in this revision.');
    } catch (error) { if (live() && generation === loadGeneration.current) setMessage(failureMessage(error)); }
    finally { if (live() && generation === loadGeneration.current) setLoading(false); }
  }
  useEffect(() => {
    let live = true; void loadContext(() => live);
    return () => { live = false; };
  }, [saved.id]);
  async function run(action: ContextRequest) {
    if (active.current || locked) return;
    active.current = true; pending.current = action; setBusy(true);
    try {
      if (action.kind === 'position') { setPosition(await saveReadingPosition(action.request)); setMessage('Reading location saved locally.'); }
      else { const note = await saveAnnotation(action.request); setNotes(list => [...list.filter(item => item.id !== note.id), note]); setBody(value => value === action.request.annotation.body ? '' : value); setMessage('Annotation saved on this exact revision.'); }
      pending.current = null;
    } catch (error) { if (isConfirmedRejection(error)) pending.current = null; setMessage(failureMessage(error)); }
    finally { active.current = false; setBusy(false); }
  }
  function remember() { if (pending.current || locked) return; return run({ kind: 'position', request: { contractVersion: 1, workspaceId, operationId: crypto.randomUUID(), expectedRevision: position?.revision ?? 0, position: { resourceId: saved.resourceId, revisionId: saved.id, blockId, offsetUtf16: 0, revision: position?.revision ?? 0 } } }); }
  function addNote() { if (pending.current || locked) return; return run({ kind: 'annotation', request: { contractVersion: 1, workspaceId, operationId: crypto.randomUUID(), expectedRevision: 0, annotation: { id: crypto.randomUUID(), resourceId: saved.resourceId, revisionId: saved.id, revision: 0, body, anchor: { resourceId: saved.resourceId, revisionId: saved.id, blockId, startUtf16: 0, endUtf16: 0, exactQuote: '', quoteHash: null } } } }); }
  return <fieldset disabled={busy || loading || locked}><legend>Saved revision {saved.revision}</legend>
    <label>Reading and annotation block<select value={blockId} onChange={event => setBlockId(event.target.value)}>{saved.document.blocks.map((block, index) => <option key={block.id} value={block.id}>Block {index + 1}: {block.content.type}</option>)}</select></label>
    <button disabled={pending.current !== null} onClick={() => void remember()}>Remember reading location</button>
    <button onClick={() => onResume({ blockId, offsetUtf16: position?.revisionId === saved.id && position.blockId === blockId ? position.offsetUtf16 : 0 })}>Resume reading here</button>
    <p role="status">{message}</p>
    <button disabled={pending.current !== null} onClick={() => void loadContext()}>Reload saved context</button>
    {pending.current && <button onClick={() => pending.current && void run(pending.current)}>Retry saved context</button>}
    <form onSubmit={event => { event.preventDefault(); void addNote(); }}><label>Annotation at start of block<textarea value={body} required maxLength={16384} onChange={event => setBody(event.target.value)} /></label><button disabled={pending.current !== null}>Add annotation</button></form>
    <ul>{notes.map(note => <li key={note.id}>{note.body} {note.revisionId !== saved.id && <strong>— attached to an older revision; reattachment is unverified</strong>}</li>)}</ul>
  </fieldset>;
}
