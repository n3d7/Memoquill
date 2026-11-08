import { useEffect, useMemo, useRef, useState, useSyncExternalStore, type FormEvent } from 'react';
import type { Document } from '../documentModel';
import type { Entity } from '../application/contentTypes';
import type { SourcePort } from '../application/ports';
import type { OriginalSource, SourcePreview } from '../application/sourceTypes';
import { LessonController } from './controller';
import { NodeEditor } from './NodeEditor';
import { LessonRenderer, type ResolvedLesson } from './LessonRenderer';
import { defaultActivity, defaults, nodeLabel, nodeTypes, Select } from './LessonForms';
import type { ActivityDefinition, Appearance, Composition, LessonNode, LessonOperation, LessonPort, RegistryDescriptor, SavedComposition } from './types';

const blankDocument = (): Document => ({ schemaVersion: 1, blocks: [{ id: crypto.randomUUID(), content: { type: 'paragraph', inlines: [] } }] });
const noResolution: ResolvedLesson = { resources: {}, activities: {}, sources: {} };
export function useResolvedLesson(port: LessonPort, composition: Composition | null, revision: string | number) {
  const [resolved, setResolved] = useState<ResolvedLesson>(noResolution); const [error, setError] = useState('');
  const pins = JSON.stringify(composition?.bindings ?? []); const unitId = composition?.unitId;
  useEffect(() => {
    let current = true; setResolved(noResolution); setError(''); if (!composition) return;
    const result: ResolvedLesson = { resources: {}, activities: {}, sources: {} };
    void Promise.all(composition.bindings.map(async binding => {
      if (binding.targetKind === 'resource') result.resources[binding.revisionId] = await port.readResource(composition.unitId, binding.revisionId);
      if (binding.targetKind === 'activity') result.activities[binding.revisionId] = await port.readActivity(composition.unitId, binding.revisionId);
      if (binding.targetKind === 'source') result.sources[binding.revisionId] = await port.readSource(composition.unitId, binding.revisionId);
    })).then(() => { if (current) setResolved(result); }, () => { if (current) setError('Some pinned lesson materials could not be read. Retry loading this lesson; its saved references are preserved.'); });
    return () => { current = false; };
  // Canonical pins plus stage revision define exactly the readable draft closure.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [port, pins, unitId, revision]);
  return { resolved, error };
}
export function LessonComposer({ workspaceId, unitId, title, port, sourcePort, materials = {} }: { workspaceId: string; unitId: string; title: string; port: LessonPort; sourcePort?: SourcePort; materials?: Record<string, Entity> }) {
  const controller = useMemo(() => new LessonController(workspaceId, unitId, port), [workspaceId, unitId, port]); const state = useSyncExternalStore(controller.subscribe, controller.getSnapshot);
  const [selected, setSelected] = useState(''); const [visited, setVisited] = useState<string[]>([]); const [mode, setMode] = useState<'edit' | 'saved'>('edit');
  const [original, setOriginal] = useState<SourcePreview | null>(null); const [sourceError, setSourceError] = useState(''); const sourceRequest = useRef(0);
  const [historical, setHistorical] = useState<SavedComposition | null>(null); const [historyBusy, setHistoryBusy] = useState(false); const historyRequest = useRef(0);
  const heading = useRef<HTMLHeadingElement>(null); const originalOpener = useRef<HTMLElement | null>(null); const composition = state.composition;
  useEffect(() => { void controller.load(); }, [controller]);
  useEffect(() => () => { sourceRequest.current++; historyRequest.current++; }, []);
  const { resolved, error: readError } = useResolvedLesson(port, composition, state.draft?.revision ?? state.saved?.id ?? '');
  function select(id: string) { setSelected(id); setVisited(old => old.includes(id) ? old : [...old, id]); }
  const node = composition?.nodes.find(n => n.nodeId === selected) ?? composition?.nodes.find(n => n.nodeId === composition.rootNodeId);
  const disabled = state.busy || state.pending || !state.draft;
  async function openSource(source: Parameters<NonNullable<Parameters<typeof LessonRenderer>[0]['onOpenSource']>>[0]) {
    const active = document.activeElement;
    originalOpener.current = active instanceof HTMLElement && heading.current?.parentElement?.contains(active) ? active : null;
    const token = ++sourceRequest.current; setOriginal(null); setSourceError('');
    if (!sourcePort) { setSourceError('The original viewer is unavailable.'); return; }
    try { const preview = await sourcePort.sourcePreview(source.sourceId); if (preview.source.digest !== source.assetDigest) throw new Error('Pinned original changed'); if (token === sourceRequest.current) setOriginal(preview); } catch { if (token === sourceRequest.current) setSourceError('The pinned original could not be opened. Its saved identity is preserved.'); }
  }
  function closeOriginal() {
    sourceRequest.current++; setOriginal(null);
    const opener = originalOpener.current; originalOpener.current = null;
    const target = opener?.isConnected && !opener.closest('[hidden]') ? opener : heading.current;
    if (target && !target.closest('[hidden]')) target.focus();
  }
  async function edit(ops: LessonOperation[]) {
    const focus = document.activeElement; const revision = controller.getSnapshot().draft?.revision;
    await controller.edit(ops);
    if (focus instanceof HTMLElement && focus.isConnected && !focus.closest('[hidden]') && (document.activeElement === document.body || document.activeElement === focus)) focus.focus();
    return controller.getSnapshot().draft?.revision !== revision && !controller.getSnapshot().pending;
  }
  async function earlierSnapshot() {
    const revision = (historical ?? state.saved)?.parentRevisionId; if (!revision || historyBusy) return;
    const token = ++historyRequest.current; setHistoryBusy(true);
    try { const snapshot = await port.readRevision(unitId, revision); if (!snapshot || snapshot.id !== revision || snapshot.unitId !== unitId) throw new Error('Unavailable snapshot'); if (token === historyRequest.current) setHistorical(snapshot); }
    catch { if (token === historyRequest.current) setSourceError('This earlier saved snapshot could not be read. The current lesson and editing draft are preserved.'); }
    finally { if (token === historyRequest.current) setHistoryBusy(false); }
  }
  return <section className="lesson-workspace" aria-label={title}>
    <h2 ref={heading} tabIndex={-1}>{title}</h2><p>Independent offline lesson · Edits and preview responses belong to this lesson.</p>
    <div className="lesson-toolbar"><button disabled={state.busy || state.pending} onClick={() => void controller.load()}>Reopen lesson draft</button>{!state.draft && <button disabled={state.busy || state.pending} onClick={() => void controller.begin()}>Edit lesson</button>}<button disabled={state.busy || state.pending || !state.canUndo} onClick={() => void controller.undo()}>Undo lesson change</button><button disabled={state.busy || state.pending || !state.canRedo} onClick={() => void controller.redo()}>Redo lesson change</button><button disabled={disabled} onClick={() => void controller.preview()}>Preview draft</button><button disabled={disabled} onClick={() => void controller.save()}>Save lesson</button></div>
    {state.drafts.length > 1 && <label>Saved editing drafts<select value={state.draft?.id ?? ''} disabled={state.busy || state.pending} onChange={e => { void controller.chooseDraft(e.target.value); setSelected(''); }} >{state.drafts.map((draft, i) => <option key={draft.id} value={draft.id}>Editing draft {i + 1}{draft.baseHeadRevision === state.saved?.id ? ' — current base' : ' — earlier base'}</option>)}</select></label>}
    <p role="status">{state.busy ? 'Working locally…' : state.message}</p>{(state.error || readError) && <div role="alert"><p>{state.error || readError}</p>{state.pending && <button disabled={state.busy} onClick={() => void controller.retry()}>Retry exact pending change</button>}<button disabled={state.busy} onClick={() => void controller.refreshHead()}>Check latest saved lesson</button>{state.conflict && <button disabled={state.busy} onClick={() => { void controller.startLatestDraft(); setSelected(''); }}>Keep old draft and start from latest saved lesson</button>}<p>Refresh keeps your draft. If its base changed, keep it for comparison and explicitly start a draft from the latest saved lesson.</p></div>}
    <nav aria-label="Lesson views"><button aria-pressed={mode === 'edit'} onClick={() => setMode('edit')}>Compose lesson</button><button disabled={!state.saved} aria-pressed={mode === 'saved'} onClick={() => setMode('saved')}>Saved lesson</button></nav>
    <div hidden={mode !== 'edit'}>{!composition && !state.busy && <p>This learning unit has no lesson composition yet. Choose Edit lesson to create an editable draft.</p>}
      {composition && <div className="lesson-authoring"><section aria-label="Lesson structure"><h3>Components in reading order</h3><Tree composition={composition} selected={node?.nodeId ?? ''} select={select} disabled={disabled} edit={edit} />{state.draft && <AddNode composition={composition} selected={node} disabled={disabled} registry={state.registry} materials={materials} sourcePort={sourcePort} add={async (operations, id) => { await edit(operations); if (controller.getSnapshot().composition?.nodes.some(n => n.nodeId === id)) select(id); }} />}</section>
      <section aria-label="Component settings"><h3>Component settings</h3>{node && !visited.includes(node.nodeId) && <button onClick={() => select(node.nodeId)}>Configure selected component</button>}{visited.map(id => { const item = composition.nodes.find(n => n.nodeId === id); return item ? <div key={id} hidden={node?.nodeId !== id}><NodeEditor node={item} composition={composition} registry={state.registry.find(d => d.typeId === item.typeId && d.typeVersion === item.typeVersion)} resolved={resolved} disabled={disabled} save={edit} /></div> : null; })}</section>
      <AppearanceForm key={unitId} appearance={composition.appearance} disabled={disabled} save={tokens => edit([{ kind: 'setAppearance', tokens }])} />
      </div>}
      {state.preview && <div className="lesson-surface" {...appearanceAttributes(state.preview.composition.appearance)}><h3>Author preview</h3>{state.preview.warnings.map((warning, i) => <p key={i}>{warning}</p>)}<LessonRenderer key={state.preview.hash} composition={state.preview.composition} {...resolved} onOpenSource={source => void openSource(source)} /></div>}
    </div>
    <div hidden={mode !== 'saved'}>{state.saved && <><SavedLesson saved={state.saved} port={port} openSource={source => void openSource(source)} />{state.saved.parentRevisionId && <button disabled={historyBusy || !(historical ?? state.saved).parentRevisionId} onClick={() => void earlierSnapshot()}>View earlier saved snapshot</button>}{historical && <aside><p>Earlier saved content is read only. Unsupported data remains inert; the current lesson and editing draft are unchanged.</p><SavedLesson saved={historical} title="Earlier saved snapshot — read only" port={port} openSource={source => void openSource(source)} /><button onClick={() => { historyRequest.current++; setHistoryBusy(false); setHistorical(null); }}>Close earlier snapshot</button></aside>}</>}</div>
    {sourceError && <p role="alert">{sourceError}</p>}{original && <aside aria-label="Preserved original"><h3>{original.source.filename}</h3><p>{original.source.attribution ?? 'Attribution unavailable'} · {original.source.license ?? 'License unknown'}</p><pre>{original.text ?? original.hex ?? 'Original bytes preserved. Text preview unavailable.'}</pre>{original.truncated && <p>Partial preview only. The original remains preserved.</p>}<button onClick={closeOriginal}>Close original preview</button></aside>}
  </section>;
}
function appearanceAttributes(appearance: Appearance) { return { 'data-theme': appearance.theme, 'data-spacing': appearance.spacing, 'data-typography': appearance.typography, 'data-width': appearance.width, 'data-emphasis': appearance.emphasis }; }
function SavedLesson({ saved, port, openSource, title = 'Saved lesson' }: { saved: NonNullable<ReturnType<LessonController['getSnapshot']>['saved']>; port: LessonPort; openSource: NonNullable<Parameters<typeof LessonRenderer>[0]['onOpenSource']>; title?: string }) {
  const { resolved, error } = useResolvedLesson(port, saved.composition, saved.id);
  return <div className="lesson-surface" {...appearanceAttributes(saved.composition.appearance)}><h3>{title}</h3>{error && <p role="alert">{error}</p>}<LessonRenderer key={saved.id} composition={saved.composition} {...resolved} onOpenSource={openSource} /></div>;
}
function Tree({ composition: c, selected, select, disabled, edit }: { composition: Composition; selected: string; select: (id: string) => void; disabled: boolean; edit: (ops: LessonOperation[]) => Promise<boolean> }) {
  const buttons = useRef(new Map<string, HTMLButtonElement>()); const previous = useRef(selected);
  useEffect(() => { if (previous.current !== selected) { const button = buttons.current.get(selected); if (button && !button.closest('[hidden]')) button.focus(); previous.current = selected; } }, [selected]);
  function render(id: string, path: string[] = [], parent?: LessonNode, slot?: string, index = 0): React.ReactNode {
    const node = c.nodes.find(n => n.nodeId === id); if (!node || path.includes(id)) return null;
    const label = nodeLabel(node, c.nodes.indexOf(node)); const siblings = parent && slot ? parent.slots[slot] : [];
    return <li key={id}><button aria-pressed={selected === id} ref={element => { if (element) buttons.current.set(id, element); else buttons.current.delete(id); }} onClick={() => select(id)}>{label}</button>{parent && slot && <><button aria-label={`Move ${label} up`} disabled={disabled || index === 0} onClick={() => void edit([{ kind: 'moveNode', id, parent: parent.nodeId, slot, index: index - 1 }])}>Up</button><button aria-label={`Move ${label} down`} disabled={disabled || index === siblings.length - 1} onClick={() => void edit([{ kind: 'moveNode', id, parent: parent.nodeId, slot, index: index + 1 }])}>Down</button><button aria-label={`Remove ${label}`} disabled={disabled} onClick={() => { void edit([{ kind: 'removeNode', id }]).then(ok => { if (ok) select(parent.nodeId); }); }}>Remove</button></>}{Object.entries(node.slots).map(([name, children]) => children.length > 0 && <div key={name}><p>{name === 'body' ? 'Contents' : name === 'primary' ? 'Primary column' : 'Secondary column'}</p><ol>{children.map((child, i) => render(child, [...path, id], node, name, i))}</ol></div>)}</li>;
  }
  return <ol className="lesson-tree">{render(c.rootNodeId)}</ol>;
}
function AddNode({ composition: c, selected, disabled, registry, materials, sourcePort, add }: { composition: Composition; selected?: LessonNode; disabled: boolean; registry: RegistryDescriptor[]; materials: Record<string, Entity>; sourcePort?: SourcePort; add: (ops: LessonOperation[], id: string) => Promise<void> }) {
  const [type, setType] = useState('section'); const [parent, setParent] = useState(''); const [slot, setSlot] = useState(''); const [family, setFamily] = useState<ActivityDefinition['family']>('recall'); const [material, setMaterial] = useState(''); const [originals, setOriginals] = useState<OriginalSource[]>([]); const [source, setSource] = useState(''); const [sourceError, setSourceError] = useState('');
  const containers = c.nodes.filter(n => n.typeVersion === 1 && ['stack', 'columns', 'section'].includes(n.typeId)); const destination = containers.find(n => n.nodeId === parent) ?? containers.find(n => n.nodeId === selected?.nodeId) ?? containers.find(n => n.nodeId === c.rootNodeId); const chosenSlot = destination && Object.hasOwn(destination.slots, slot) ? slot : destination ? Object.keys(destination.slots)[0] : '';
  useEffect(() => { let current = true; setOriginals([]); setSource(''); if (!material || !sourcePort) return; void sourcePort.sources(material).then(result => { if (current) { setOriginals(result.sources); setSourceError(''); } }, () => { if (current) setSourceError('Originals could not be listed.'); }); return () => { current = false; }; }, [material, sourcePort]);
  async function submit(e: FormEvent) {
    e.preventDefault(); if (!destination) return; const id = crypto.randomUUID(); const bindingId = crypto.randomUUID(); const props = structuredClone(defaults(type)); const operations: LessonOperation[] = [];
    if (type === 'content') { operations.push({ kind: 'createResourceRevision', resourceId: crypto.randomUUID(), expectedRevisionId: null, title: 'Lesson material', bindingId, document: blankDocument() }); Object.assign(props, { bindingId }); }
    if (type === 'activity') { const definition = defaultActivity(family); if (definition.family === 'singleChoice') definition.correctOptionId = definition.options[0].id; operations.push({ kind: 'createActivityRevision', activityId: crypto.randomUUID(), expectedRevisionId: null, bindingId, definition }); Object.assign(props, { bindingId }); }
    if (type === 'source') { if (!source) return; operations.push({ kind: 'createSourceBindingRevision', sourceId: source, bindingId, declaredUsedRanges: [] }); Object.assign(props, { bindingId }); }
    if (type === 'button') Object.assign(props, { label: 'Focus lesson', action: { actionId: 'view.focusNode', version: 1, targetNodeId: c.rootNodeId, inputs: {} } });
    const descriptor = registry.find(d => d.typeId === type); const slots = descriptor?.slots ?? (type === 'columns' ? ['primary', 'secondary'] : ['stack', 'section'].includes(type) ? ['body'] : []);
    operations.push({ kind: 'insertNode', parent: destination.nodeId, slot: chosenSlot, index: destination.slots[chosenSlot].length, node: { nodeId: id, typeId: type, typeVersion: 1, props, slots: Object.fromEntries(slots.map(slot => [slot, []])), on: {} } }); await add(operations, id);
  }
  return <form onSubmit={e => void submit(e)}><fieldset disabled={disabled}><legend>Add component</legend><Select label="Component type" value={type} values={nodeTypes} onChange={setType} /><label>Add inside<select value={destination?.nodeId ?? ''} onChange={e => { setParent(e.target.value); setSlot(''); }}>{containers.map(n => <option key={n.nodeId} value={n.nodeId}>{nodeLabel(n, c.nodes.indexOf(n))}</option>)}</select></label>{destination && <Select label="Destination area" value={chosenSlot} values={Object.keys(destination.slots)} onChange={setSlot} />}{type === 'activity' && <Select label="New exercise family" value={family} values={['recall', 'singleChoice', 'constructed', 'procedure']} onChange={value => setFamily(value as ActivityDefinition['family'])} />}{type === 'source' && <><label>Material with original<select value={material} onChange={e => setMaterial(e.target.value)}><option value="">Choose material</option>{Object.values(materials).filter(m => !m.archived).map(m => <option key={m.id} value={m.id}>{m.title}</option>)}</select></label><label>Preserved original<select value={source} onChange={e => setSource(e.target.value)}><option value="">Choose original</option>{originals.map(s => <option key={s.id} value={s.id}>{s.filename}</option>)}</select></label>{sourceError && <p role="alert">{sourceError}</p>}<p>Attach originals through the material's source controls first. No automatic extraction occurs.</p></>}<button disabled={!destination || type === 'source' && !source}>Add component</button></fieldset></form>;
}
function AppearanceForm({ appearance, disabled, save }: { appearance: Appearance; disabled: boolean; save: (tokens: Appearance) => Promise<boolean> }) {
  const [tokens, set] = useState(appearance); return <form onSubmit={e => { e.preventDefault(); void save(tokens); }}><fieldset disabled={disabled}><legend>Appearance for this lesson</legend>{Object.entries({ spacing: ['compact', 'normal', 'wide'], theme: ['system', 'light', 'dark'], typography: ['default', 'serif'], width: ['normal', 'wide'], emphasis: ['normal', 'strong'] }).map(([name, values]) => <Select key={name} label={`Lesson ${name}`} value={tokens[name as keyof Appearance]} values={values} onChange={value => set({ ...tokens, [name]: value })} />)}<button>Save lesson appearance</button></fieldset></form>;
}
