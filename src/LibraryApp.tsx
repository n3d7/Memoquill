import { useEffect, useId, useRef, useState, type FormEvent } from 'react';
import { useContentLibrary } from './useContentLibrary';
import { GoalForm, NameForm, Options, PathForms, ReorderControls, UnitForms } from './OrganizationForms';
import { MaterialPanel } from './MaterialPanel';
import { ReadingPreferencesPanel } from './ReadingPreferencesPanel';
import { useReadingPreferences } from './useReadingPreferences';
import { failureMessage, type Collection, type Command, type Entity, type Library, type SearchPage } from './contentBridge';

import { useContentPort, useIdentityPort, useOptionalLessonPort, useSourcePort } from './application/ApplicationProvider';
import { LessonTabs } from './lessons/LessonTabs';
import { StatusMessage } from './application/controls';

type Perform = (command: Command) => Promise<Library | null>;
type Content = ReturnType<typeof useContentLibrary>;
export function LibraryApp() {
  const mainId = useId();
  const lessonPort = useOptionalLessonPort(); const sourcePort = useSourcePort();
  const content = useContentLibrary(); const reading = useReadingPreferences();
  const [subjectId, setSubjectId] = useState(''); const [collectionId, setCollectionId] = useState<string | null>(null);
  const [resourceId, setResourceId] = useState(''); const [dirty, setDirty] = useState(false);
  const [notice, setNotice] = useState('');
  const data = content.snapshot;
  const identity = useIdentityPort();
  const session = data ? identity.session(data.workspaceId) : null;
  function openResource(id: string) { if (dirty && id !== resourceId) { setNotice('Save or discard the current draft before opening another material.'); return; } setResourceId(id); setNotice(''); }
  return <div className="app" data-theme={reading.preferences.theme}>
    <a className="skip-link" href={`#${mainId}`}>Skip to workspace</a><header><p className="eyebrow">Your local learning workspace</p><h1>Memoquill</h1><p>Create subjects, organize shared materials and author your own learning paths.</p></header>
    <p aria-label="Local identity">{session ? 'Local guest · Offline workspace' : 'Local guest'}</p>
    <ReadingPreferencesPanel reading={reading} />
    <main id={mainId} tabIndex={-1}>
      <OrganizationStatus content={content} />
      {notice && <p role="alert">{notice}</p>}
      <SubjectNavigation content={content} selected={subjectId} select={id => { setSubjectId(id); setCollectionId(null); }} />
      {data && <LibraryContents data={data} content={content} subjectId={subjectId} collectionId={collectionId} selectCollection={setCollectionId} resourceId={resourceId} openResource={openResource} onDirty={setDirty} />}
    {data && lessonPort && <LessonTabs data={data} port={lessonPort} sources={sourcePort} perform={content.perform} />}
    </main>
  </div>;
}
function OrganizationStatus({ content }: { content: Content }) {
  return <><StatusMessage>{content.busy ? 'Saving organization…' : content.message}</StatusMessage>{(content.error || content.hasPending) && <div role="alert"><p>{content.message}</p><button onClick={() => void content.retry()}>Retry last change</button><button onClick={() => void content.reload()}>Reload organization</button></div>}</>;
}
function SubjectNavigation({ content, selected, select }: { content: Content; selected: string; select: (id: string) => void }) {
  const subjects = Object.values(content.snapshot?.subjects ?? {}).filter(subject => !subject.archived);
  async function create(title: string) { const id = crypto.randomUUID(); const result = await content.perform({ type: 'createSubject', id, title, priorKnowledge: null }); if (result) select(id); return result; }
  return <fieldset disabled={content.busy || content.hasPending || !content.snapshot}><legend>Subjects</legend><NameForm label="New subject" action="Create subject" onSubmit={create} /><nav aria-label="Subjects"><ul>{subjects.map(subject => <li key={subject.id}><button aria-current={selected === subject.id ? 'page' : undefined} onClick={() => select(subject.id)}>{subject.title}</button></li>)}</ul></nav>{content.snapshot && !subjects.length && <p>No active subjects. Create a subject to begin.</p>}</fieldset>;
}
function LibraryContents({ data, content, subjectId, collectionId, selectCollection, resourceId, openResource, onDirty }: { data: Library; content: Content; subjectId: string; collectionId: string | null; selectCollection: (id: string | null) => void; resourceId: string; openResource: (id: string) => void; onDirty: (dirty: boolean) => void }) {
  async function createMaterial(title: string) { const id = crypto.randomUUID(); const result = await content.perform({ type: 'createResource', id, title }); if (result) openResource(id); return result; }
  return <>{subjectId && <fieldset disabled={content.busy || content.hasPending}><legend>Selected subject</legend><SubjectPanel data={data} subjectId={subjectId} collectionId={collectionId} selectCollection={selectCollection} perform={content.perform} openResource={openResource} /></fieldset>}
    <fieldset disabled={content.busy || content.hasPending}><legend>Materials</legend><NameForm label="New material" action="Create material" onSubmit={createMaterial} /><MaterialList data={data} selected={resourceId} open={openResource} /></fieldset>
    {data.resources[resourceId] && <MaterialPanel key={resourceId} data={data} resourceId={resourceId} onDirty={onDirty} perform={content.perform} onLibraryChanged={() => void content.reload()} />}
    <fieldset disabled={content.busy || content.hasPending}><legend>Units and paths</legend><UnitForms data={data} perform={content.perform} /><PathForms data={data} perform={content.perform} /></fieldset>
    <fieldset disabled={content.busy || content.hasPending}><legend>Archive and restore</legend><ArchivePanel data={data} perform={content.perform} /></fieldset></>;
}
function SubjectPanel({ data, subjectId, collectionId, selectCollection, perform, openResource }: { data: Library; subjectId: string; collectionId: string | null; selectCollection: (id: string | null) => void; perform: Perform; openResource: (id: string) => void }) {
  const subject = data.subjects[subjectId];
  return <fieldset disabled={subject.archived}><legend>{subject.title}</legend><p>Current subject: {subject.title}{subject.archived ? ' (archived)' : ''}</p><GoalForm key={subjectId} subjectId={subjectId} perform={perform} /><ul>{Object.values(data.goals).filter(goal => goal.subjectId === subjectId && !goal.archived).map(goal => <li key={goal.id}>{goal.title}{goal.criteria && <p>{goal.criteria}</p>}</li>)}</ul><CollectionPanel data={data} subjectId={subjectId} selected={collectionId} select={selectCollection} perform={perform} openResource={openResource} /></fieldset>;
}
function useMaterialSearch(libraryRevision: number) {
  const { searchResources, rebuildSearchIndex } = useContentPort();
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<SearchPage | null>(null); const [busy, setBusy] = useState(false); const [error, setError] = useState('');
  const request = useRef(0); const revision = useRef(libraryRevision); revision.current = libraryRevision;
  useEffect(() => { request.current += 1; setResult(null); setBusy(false); setError(''); }, [libraryRevision]);
  useEffect(() => () => { request.current += 1; }, []);
  async function search(offset = 0) {
    const token = ++request.current; const expected = libraryRevision;
    setBusy(true); setError('');
    try {
      const value = await searchResources(query, offset);
      if (token !== request.current || revision.current !== expected) return;
      if (value.revision !== expected) { setResult(null); setError('The library changed. Reload organization before searching again.'); return; }
      setResult(value);
    } catch (reason) { if (token === request.current) setError(failureMessage(reason)); }
    finally { if (token === request.current) setBusy(false); }
  }
  function edit(value: string) { request.current += 1; setQuery(value); setResult(null); setBusy(false); setError(''); }
  async function rebuild() { if (!rebuildSearchIndex) return; const token = ++request.current; setBusy(true); setError(''); try { await rebuildSearchIndex(); if (token === request.current) setResult(null); } catch (reason) { if (token === request.current) setError(failureMessage(reason)); } finally { if (token === request.current) setBusy(false); } }
  return { query, result, busy, error, search, edit, rebuild, canRebuild: !!rebuildSearchIndex };
}
type MaterialSearch = ReturnType<typeof useMaterialSearch>;
function MaterialList({ data, selected, open }: { data: Library; selected: string; open: (id: string) => void }) {
  const search = useMaterialSearch(data.revision); const [page, setPage] = useState(0);
  const heading = useRef<HTMLHeadingElement>(null); const focus = useRef(false);
  useEffect(() => { if (focus.current) { heading.current?.focus(); focus.current = false; } }, [search.result, page]);
  const result = search.result;
  const resources = Object.values(data.resources).filter(item => !item.archived);
  const lastPage = Math.max(0, Math.ceil(resources.length / 20) - 1); const currentPage = Math.min(page, lastPage);
  const visible = materialResults(result, resources, currentPage);
  return <section aria-label="Material browser">
    <SearchForm search={search} beforeSearch={() => { focus.current = true; }} />
    {search.canRebuild && <button disabled={search.busy} onClick={() => void search.rebuild()}>Rebuild search index</button>}
    <h3 ref={heading} tabIndex={-1}>{result ? 'Search results' : 'Available materials'}</h3>
    {result ? <><SearchStatus result={result} />{result.indexRebuilt && <p role="status">The search index was rebuilt from saved materials.</p>}</> : <MaterialCount count={resources.length} page={currentPage} />}
    <ul>{visible.map(item => <li key={item.id}><button aria-current={selected === item.id ? 'page' : undefined} onClick={() => open(item.id)}>{item.title}</button>{item.snippet && <p>{item.snippet}</p>}{'references' in item && item.references?.map((ref, index) => <p key={`${ref.sourceId}-${index}`}>Original {ref.sourceId}, {ref.locator.kind} {ref.locator.value}{!ref.originalAvailable && ' — original unavailable or corrupt'}</p>)}</li>)}</ul>
    {result ? <ContinueSearch search={search} beforeSearch={() => { focus.current = true; }} /> : <div className="actions"><button disabled={!currentPage} onClick={() => { focus.current = true; setPage(currentPage - 1); }}>Previous materials</button><button disabled={currentPage >= lastPage} onClick={() => { focus.current = true; setPage(currentPage + 1); }}>Next materials</button></div>}
  </section>;
}
function materialResults(result: SearchPage | null, resources: Entity[], page: number) {
  if (result) return result.items.map(item => ({ id: item.resourceId, title: item.title, snippet: item.snippet, references: item.references }));
  return resources.slice(page * 20, page * 20 + 20).map(item => ({ ...item, snippet: '' }));
}
function SearchForm({ search, beforeSearch }: { search: MaterialSearch; beforeSearch: () => void }) {
  const helpId = useId();
  const valid = search.query.length > 0 && [...search.query].length <= 256;
  function submit(event: FormEvent) { event.preventDefault(); if (valid) { beforeSearch(); void search.search(); } }
  return <><form onSubmit={submit}><label>Search titles, tags and text<input type="search" value={search.query} onChange={event => search.edit(event.target.value)} aria-describedby={helpId} /></label><p id={helpId}>Literal text, up to 256 characters. Searches saved titles, tags and canonical text in bounded batches.</p><button disabled={search.busy || !valid}>Search materials</button><button type="button" onClick={() => search.edit('')}>Show all materials</button></form>{[...search.query].length > 256 && <p role="alert">Use at most 256 characters.</p>}{search.error && <p role="alert">{search.error}</p>}{search.busy && <p role="status">Searching saved materials…</p>}</>;
}
function ContinueSearch({ search, beforeSearch }: { search: MaterialSearch; beforeSearch: () => void }) {
  const offset = search.result?.nextOffset ?? null;
  return <button disabled={search.busy || offset === null} onClick={() => { if (offset !== null) { beforeSearch(); void search.search(offset); } }}>Continue search</button>;
}
function MaterialCount({ count, page }: { count: number; page: number }) {
  return <p role="status">{count ? `${count} materials. Showing ${page * 20 + 1}–${Math.min((page + 1) * 20, count)}.` : 'No active materials. Create a material to begin.'}</p>;
}
function SearchStatus({ result }: { result: SearchPage }) {
  if (result.items.length) return <p role="status">{result.items.length} matches in this batch.{result.nextOffset !== null ? ' More materials remain to search.' : ' Search complete.'}</p>;
  return <p role="status">{result.nextOffset === null ? 'No matches in this batch. Search complete.' : 'No matches in this batch. More materials remain to search.'}</p>;
}
function ArchivePanel({ data, perform }: { data: Library; perform: Perform }) {
  const groups = [['subject', data.subjects], ['goal', data.goals], ['collection', data.collections], ['resource', data.resources], ['unit', data.units], ['path', data.paths]] as const;
  return <details><summary>Manage archived items</summary><p>Archiving preserves the item and its identity. Restore an item to make it available again.</p>{groups.map(([kind, items]) => <ArchiveGroup key={kind} kind={kind} items={items} perform={perform} />)}<ul>{Object.values(data.placements).filter(item => item.archived).map(item => <li key={item.id}>Removed placement in {data.collections[item.collectionId]?.title}<button aria-label={`Restore placement ${item.id}`} onClick={() => void perform({ type: 'archive', target: { kind: 'placement', id: item.id }, archived: false })}>Restore placement</button></li>)}</ul></details>;
}
function ArchiveGroup({ kind, items, perform }: { kind: 'subject' | 'goal' | 'collection' | 'resource' | 'unit' | 'path'; items: Record<string, Entity>; perform: Perform }) {
  const label = kind === 'resource' ? 'material' : kind;
  return <section><h3>{label.charAt(0).toUpperCase() + label.slice(1)} archive controls</h3><ul>{Object.values(items).map(item => <li key={item.id}>{item.title}{item.archived ? ' (archived)' : ''}<button aria-label={`${item.archived ? 'Restore' : 'Archive'} ${label} ${item.title}`} onClick={() => void perform({ type: 'archive', target: { kind, id: item.id }, archived: !item.archived })}>{item.archived ? 'Restore' : 'Archive'}</button></li>)}</ul></section>;
}
function CollectionPanel({ data, subjectId, selected, select, perform, openResource }: { data: Library; subjectId: string; selected: string | null; select: (id: string | null) => void; perform: Perform; openResource: (id: string) => void }) {
  const current = selected ? data.collections[selected] : null;
  const children = Object.values(data.collections).filter(x => x.subjectId === subjectId && x.parentId === selected && !x.archived).sort((a, b) => a.position - b.position);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => { heading.current?.focus(); }, [selected, subjectId]);
  return <section><h3 ref={heading} tabIndex={-1}>Collections {current ? `— ${current.title}` : '— subject root'}</h3>
    {current && <button onClick={() => select(current.parentId)}>Up to parent</button>}
    <NameForm label="New collection" action="Create collection" onSubmit={title => perform({ type: 'createCollection', id: crypto.randomUUID(), subjectId, parentId: selected, title })} />
    <ul>{children.map(item => <li key={item.id}><button onClick={() => select(item.id)}>{item.title}</button> <ReorderCollection item={item} siblings={children} perform={perform} /></li>)}</ul>
    {!children.length && <p>No active collections at this location.</p>}
    {current && <CollectionActions key={current.id} current={current} data={data} select={select} perform={perform} openResource={openResource} />}
  </section>;
}
function CollectionActions({ current, data, select, perform, openResource }: { current: Collection; data: Library; select: (id: string | null) => void; perform: Perform; openResource: (id: string) => void }) {
  const [targetSubject, setTargetSubject] = useState(''); const [targetParent, setTargetParent] = useState('');
  const [resource, setResource] = useState(''); const [unit, setUnit] = useState('');
  const placements = Object.values(data.placements).filter(x => x.collectionId === current.id && !x.archived).sort((a, b) => a.position - b.position);
  async function move() { const result = await perform({ type: 'moveCollection', id: current.id, subjectId: targetSubject, parentId: targetParent || null }); if (result) select(null); }
  return <>
      <NameForm label="New collection title" action="Rename collection" onSubmit={title => perform({ type: 'renameCollection', id: current.id, title })} />
      <label>Move collection to subject<select value={targetSubject} onChange={e => { setTargetSubject(e.target.value); setTargetParent(''); }}><option value="">Choose subject</option><Options items={data.subjects} /></select></label>
      <label>Destination parent<select value={targetParent} onChange={e => setTargetParent(e.target.value)}><option value="">Subject root</option>{Object.values(data.collections).filter(x => x.subjectId === targetSubject && !x.archived).map(x => <option key={x.id} value={x.id}>{x.title}</option>)}</select></label>
      <button disabled={!targetSubject} onClick={() => void move()}>Move collection</button>
      <button onClick={() => void perform({ type: 'archive', target: { kind: 'collection', id: current.id }, archived: true }).then(result => { if (result) select(null); })}>Archive collection</button>
      <label>Reuse material<select value={resource} onChange={e => setResource(e.target.value)}><option value="">Choose material</option><Options items={data.resources} /></select></label>
      <button disabled={!resource} onClick={() => void perform({ type: 'place', id: crypto.randomUUID(), collectionId: current.id, target: { kind: 'resource', id: resource } })}>Add shared placement</button>
      <label>Place learning unit<select value={unit} onChange={e => setUnit(e.target.value)}><option value="">Choose unit</option><Options items={data.units} /></select></label>
      <button disabled={!unit} onClick={() => void perform({ type: 'place', id: crypto.randomUUID(), collectionId: current.id, target: { kind: 'unit', id: unit } })}>Place unit</button>
      <ol>{placements.map(item => <PlacementRow key={item.id} item={item} data={data} siblings={placements} perform={perform} openResource={openResource} />)}</ol>
      {!placements.length && <p>No shared placements in this collection.</p>}
    </>;
}
function ReorderCollection({ item, siblings, perform }: { item: Collection; siblings: Collection[]; perform: Perform }) {
  return <ReorderControls title={item.title} id={item.id} ids={siblings.map(x => x.id)} reorder={orderedIds => void perform({ type: 'reorderCollections', subjectId: item.subjectId, parentId: item.parentId, orderedIds })} />;
}
function PlacementRow({ item, data, siblings, perform, openResource }: { item: import('./contentBridge').Placement; data: Library; siblings: import('./contentBridge').Placement[]; perform: Perform; openResource: (id: string) => void }) {
  const [destination, setDestination] = useState('');
  const title = item.target.kind === 'resource' ? data.resources[item.target.id]?.title : data.units[item.target.id]?.title;
  return <li>{item.target.kind === 'resource' ? <button onClick={() => openResource(item.target.id)}>{title}</button> : title}
    <ReorderControls title={`placement of ${title}`} id={item.id} ids={siblings.map(x => x.id)} reorder={orderedIds => void perform({ type: 'reorderPlacements', collectionId: item.collectionId, orderedIds })} />
    <label>Move placement of {title} to collection<select value={destination} onChange={e => setDestination(e.target.value)}><option value="">Choose collection</option><Options items={data.collections} /></select></label>
    <button disabled={!destination} onClick={() => void perform({ type: 'movePlacement', id: item.id, collectionId: destination })}>Move placement</button>
    <button onClick={() => void perform({ type: 'archive', target: { kind: 'placement', id: item.id }, archived: true })}>Remove placement</button>
  </li>;
}
