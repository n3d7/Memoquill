import { EntitySelect } from './application/EntitySelect';
import { useKeyboardShortcuts } from './settings/shortcuts';
import { StudySettingsProvider } from './study/sessionSettings';
import { useEffect, useId, useMemo, useRef, useState, type FormEvent } from 'react';
import { useContentLibrary } from './useContentLibrary';
import { GoalForm, NameForm, Options, PathForms, ReorderControls, UnitForms } from './OrganizationForms';
import { MaterialPanel } from './MaterialPanel';
import { SettingsPanel, useSettings } from './settings/SettingsPanel';
import { ReadingPreferencesPanel } from './ReadingPreferencesPanel';
import { useReadingPreferences } from './useReadingPreferences';
import { failureMessage, type Collection, type Command, type Entity, type Library, type SearchPage } from './contentBridge';

import { useContentPort, useIdentityPort, useOptionalLessonPort, useSourcePort } from './application/ApplicationProvider';
import { LessonTabs } from './lessons/LessonTabs';
import { CourseDraftReview } from './courses/CourseDraftReview';
import { StatusMessage } from './application/controls';
import type { NavigationContext } from './application/workspaceTypes';

type Perform = (command: Command) => Promise<Library | null>;
type Content = ReturnType<typeof useContentLibrary>;
export function LibraryApp({ reviewCourseId }: { reviewCourseId?: string } = {}) {
  const mainId = useId();
  const lessonPort = useOptionalLessonPort(); const sourcePort = useSourcePort();
  const content = useContentLibrary(); const reading = useReadingPreferences();
  const [subjectId, setSubjectId] = useState(''); const [collectionId, setCollectionId] = useState<string | null>(null);
  const selectedSubject = content.snapshot?.subjects[subjectId];
  const settings = useSettings(reading, selectedSubject && !selectedSubject.archived ? subjectId : '');
  const appRef = useRef<HTMLDivElement>(null); const settingsRef = useRef<HTMLDetailsElement>(null);
  useKeyboardShortcuts(reading.preferences.shortcuts, {
    openSettings: () => { const node = settingsRef.current; if (!node) return false; node.open = true; node.querySelector('summary')?.focus(); return true; },
    focusSearch: () => { const input = appRef.current?.querySelector<HTMLInputElement>('input[type="search"]'); if (!input || input.disabled || input.closest('[hidden]')) return false; input.focus(); return true; },
    focusWorkspace: () => { const node = document.getElementById(mainId); if (!node) return false; node.focus(); return true; },
  });
  const [resourceId, setResourceId] = useState(''); const [dirty, setDirty] = useState(false);
  const [notice, setNotice] = useState('');
  const [initialNavigation, setInitialNavigation] = useState<NavigationContext | null>(null);
  const browserContext = useRef({ query: '', page: 0, searchOffset: 0, searchActive: false });
  const updateBrowserContext = (part: Partial<NavigationContext>) => { Object.assign(browserContext.current, part); };
  async function rememberContext() {
    const context: NavigationContext = { contractVersion: 1, subjectId, collectionId, resourceId, ...browserContext.current };
    setNotice(await reading.saveNavigation(context));
  }
  function restoreContext() {
    const context = reading.preferences.navigation;
    if (!context || !data || dirty || content.busy || content.hasPending) return;
    const subject = data.subjects[context.subjectId];
    const subjectId = subject && !subject.archived ? context.subjectId : '';
    const collection = context.collectionId ? data.collections[context.collectionId] : null;
    setSubjectId(subjectId); setCollectionId(collection && !collection.archived && collection.subjectId === subjectId ? context.collectionId : null);
    const resource = data.resources[context.resourceId];
    setResourceId(resource && !resource.archived ? context.resourceId : '');
    Object.assign(browserContext.current, context);
    setInitialNavigation({ ...context });
    setNotice('Remembered workspace context restored. Missing or archived selections are left closed; search is rebuilt from current saved data.');
    document.getElementById(mainId)?.focus();
  }
  const data = content.snapshot;
  const identity = useIdentityPort();
  const session = data ? identity.session(data.workspaceId) : null;
  function openResource(id: string) { if (dirty && id !== resourceId) { setNotice('Save or discard the current draft before opening another material.'); return; } setResourceId(id); setNotice(''); }
  return <StudySettingsProvider settings={reading.loaded && (!settings.workspace?.available || settings.workspace.loaded) ? settings.effective : null}><div ref={appRef} className="app" data-theme={settings.effective.theme}>
    <a className="skip-link" href={`#${mainId}`}>Skip to workspace</a><header><p className="eyebrow">Your local learning workspace</p><h1>Memoquill</h1><p>Create subjects, organize shared materials and author your own learning paths.</p></header>
    <p aria-label="Local identity">{session ? 'Local guest · Offline workspace' : 'Local guest'}</p>
    <ReadingPreferencesPanel reading={reading} effective={settings.effective} /><SettingsPanel reading={reading} settings={settings} detailsRef={settingsRef} subjects={data?.subjects} />
    <section aria-label="Workspace continuity"><h2>Remembered workspace context</h2>
      <button disabled={!data || !reading.loaded || reading.working || reading.retry} onClick={() => void rememberContext()}>Remember workspace context</button>
      <button disabled={!data || !reading.preferences.navigation || reading.working || reading.retry || dirty || content.busy || content.hasPending} onClick={restoreContext}>Restore remembered workspace context</button>
      <button disabled={!reading.preferences.navigation || reading.working || reading.retry} onClick={() => { void reading.saveNavigation(null).then(setNotice); }}>Forget remembered workspace context</button>
      <p>Stores selection, query and page for this course. Reading positions stay attached to saved material revisions. Restore keeps unsaved material drafts protected.</p>
    </section>
    <main id={mainId} tabIndex={-1}>
      <OrganizationStatus content={content} />
    {reviewCourseId && <CourseDraftReview courseId={reviewCourseId} onChanged={content.reload} />}
      {notice && <p role="alert">{notice}</p>}
      <SubjectNavigation content={content} selected={subjectId} select={id => { setSubjectId(id); setCollectionId(null); }} />
      {data && <LibraryContents data={data} content={content} subjectId={subjectId} collectionId={collectionId} selectCollection={setCollectionId} initialNavigation={initialNavigation} updateBrowserContext={updateBrowserContext} resourceId={resourceId} openResource={openResource} onDirty={setDirty} />}
    {data && lessonPort && <LessonTabs data={data} port={lessonPort} sources={sourcePort} perform={content.perform} />}
    </main>
  </div></StudySettingsProvider>;
}
function OrganizationStatus({ content }: { content: Content }) {
  return <><StatusMessage>{content.busy ? 'Saving organization…' : content.message}</StatusMessage>{(content.error || content.hasPending) && <div role="alert"><p>{content.message}</p><button onClick={() => void content.retry()}>Retry last change</button><button onClick={() => void content.reload()}>Reload organization</button></div>}</>;
}
function SubjectNavigation({ content, selected, select }: { content: Content; selected: string; select: (id: string) => void }) {
  const subjects = Object.values(content.snapshot?.subjects ?? {}).filter(subject => !subject.archived);
  async function create(title: string) { const id = crypto.randomUUID(); const result = await content.perform({ type: 'createSubject', id, title, priorKnowledge: null }); if (result) select(id); return result; }
  return <fieldset disabled={content.busy || content.hasPending || !content.snapshot}><legend>Subjects</legend><NameForm label="New subject" action="Create subject" onSubmit={create} /><nav aria-label="Subjects"><ul>{subjects.map(subject => <li key={subject.id}><button aria-current={selected === subject.id ? 'page' : undefined} onClick={() => select(subject.id)}>{subject.title}</button></li>)}</ul></nav>{content.snapshot && !subjects.length && <p>No active subjects. Create a subject to begin.</p>}</fieldset>;
}
function LibraryContents({ data, content, subjectId, collectionId, selectCollection, resourceId, openResource, onDirty, initialNavigation, updateBrowserContext }: { data: Library; content: Content; subjectId: string; collectionId: string | null; selectCollection: (id: string | null) => void; resourceId: string; openResource: (id: string) => void; onDirty: (dirty: boolean) => void; initialNavigation: NavigationContext | null; updateBrowserContext: (part: Partial<NavigationContext>) => void }) {
  async function createMaterial(title: string) { const id = crypto.randomUUID(); const result = await content.perform({ type: 'createResource', id, title }); if (result) openResource(id); return result; }
  return <>{subjectId && <fieldset disabled={content.busy || content.hasPending}><legend>Selected subject</legend><SubjectPanel data={data} subjectId={subjectId} collectionId={collectionId} selectCollection={selectCollection} perform={content.perform} openResource={openResource} /></fieldset>}
    <fieldset disabled={content.busy || content.hasPending}><legend>Materials</legend><NameForm label="New material" action="Create material" onSubmit={createMaterial} /><MaterialList data={data} selected={resourceId} open={openResource} initialNavigation={initialNavigation} onContext={updateBrowserContext} /></fieldset>
    {data.resources[resourceId] && <MaterialPanel key={resourceId} data={data} resourceId={resourceId} onDirty={onDirty} perform={content.perform} onLibraryChanged={() => void content.reload()} />}
    <fieldset disabled={content.busy || content.hasPending}><legend>Units and paths</legend><UnitForms data={data} perform={content.perform} /><PathForms data={data} perform={content.perform} /></fieldset>
    <fieldset disabled={content.busy || content.hasPending}><legend>Archive and restore</legend><ArchivePanel data={data} perform={content.perform} /></fieldset></>;
}
function SubjectPanel({ data, subjectId, collectionId, selectCollection, perform, openResource }: { data: Library; subjectId: string; collectionId: string | null; selectCollection: (id: string | null) => void; perform: Perform; openResource: (id: string) => void }) {
  const subject = data.subjects[subjectId];
  return <fieldset disabled={subject.archived}><legend>{subject.title}</legend><p>Current subject: {subject.title}{subject.archived ? ' (archived)' : ''}</p><GoalForm key={subjectId} subjectId={subjectId} perform={perform} /><ul>{Object.values(data.goals).filter(goal => goal.subjectId === subjectId && !goal.archived).map(goal => <li key={goal.id}>{goal.title}{goal.criteria && <p>{goal.criteria}</p>}</li>)}</ul><CollectionPanel data={data} subjectId={subjectId} selected={collectionId} select={selectCollection} perform={perform} openResource={openResource} /></fieldset>;
}
function useMaterialSearch(libraryRevision: number, onContext: (part: Partial<NavigationContext>) => void) {
  const { searchResources, rebuildSearchIndex } = useContentPort();
  const [query, setQuery] = useState('');
  const [result, setResult] = useState<SearchPage | null>(null);
  const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const [scannedBatches, setScannedBatches] = useState(0);
  const request = useRef(0); const revision = useRef(libraryRevision); revision.current = libraryRevision;
  useEffect(() => { request.current += 1; setResult(null); setScannedBatches(0); setBusy(false); setError(''); }, [libraryRevision]);
  useEffect(() => () => { request.current += 1; }, []);
  async function search(offset = 0, restoredQuery?: string) {
    const currentQuery = restoredQuery ?? query;
    const token = ++request.current; const expected = libraryRevision;
    setBusy(true); setError(''); setScannedBatches(0);
    try {
      let cursor = offset; const seen = new Set([cursor]);
      for (let batch = 0; batch < 64; batch += 1) {
        const value = await searchResources(currentQuery, cursor);
        if (token !== request.current || revision.current !== expected) return;
        if (value.revision !== expected) { setResult(null); setError('The library changed. Reload organization before searching again.'); return; }
        if (value.nextOffset !== null && (!Number.isSafeInteger(value.nextOffset) || value.nextOffset < 0 || seen.has(value.nextOffset))) {
          setResult(null); setError('Search could not continue. Try searching again.'); return;
        }
        if (value.items.length || value.nextOffset === null || batch === 63) {
          setResult(value); onContext({ query: currentQuery, searchOffset: cursor, searchActive: true }); return;
        }
        seen.add(value.nextOffset); cursor = value.nextOffset; setScannedBatches(batch + 1);
      }
    } catch (reason) { if (token === request.current) setError(failureMessage(reason)); }
    finally { if (token === request.current) { setBusy(false); setScannedBatches(0); } }
  }
  function edit(value: string) { onContext({ query: value, searchOffset: 0, searchActive: false }); request.current += 1; setQuery(value); setResult(null); setScannedBatches(0); setBusy(false); setError(''); }
  async function rebuild() { if (!rebuildSearchIndex) return; const token = ++request.current; setBusy(true); setError(''); setScannedBatches(0); try { await rebuildSearchIndex(); if (token === request.current) setResult(null); } catch (reason) { if (token === request.current) setError(failureMessage(reason)); } finally { if (token === request.current) setBusy(false); } }
  return { restore: (context: NavigationContext) => { edit(context.query); if (context.searchActive) void search(context.searchOffset, context.query); }, query, result, scannedBatches, busy, error, search, edit, rebuild, canRebuild: !!rebuildSearchIndex };
}
type MaterialSearch = ReturnType<typeof useMaterialSearch>;
function MaterialList({ data, selected, open, initialNavigation, onContext }: { data: Library; selected: string; open: (id: string) => void; initialNavigation: NavigationContext | null; onContext: (part: Partial<NavigationContext>) => void }) {
  const search = useMaterialSearch(data.revision, onContext); const [page, setPage] = useState(0);
  useEffect(() => { if (initialNavigation) { setPage(initialNavigation.page); search.restore(initialNavigation); } }, [initialNavigation]);
  const heading = useRef<HTMLHeadingElement>(null); const focus = useRef(false); const materialStatusId = useId();
  useEffect(() => { if (focus.current) { heading.current?.focus(); focus.current = false; } }, [search.result, page]);
  const result = search.result;
  const resources = Object.values(data.resources).filter(item => !item.archived);
  const lastPage = Math.max(0, Math.ceil(resources.length / 20) - 1); const currentPage = Math.min(page, lastPage);
  const visible = materialResults(result, resources, currentPage);
  return <section aria-label="Material browser">
    <SearchForm search={search} beforeSearch={() => { focus.current = true; }} beforeEdit={() => { focus.current = false; }} />
    {search.canRebuild && <button disabled={search.busy} onClick={() => void search.rebuild()}>Rebuild search index</button>}
    <h3 ref={heading} tabIndex={-1} aria-describedby={materialStatusId}>{result ? 'Search results' : 'Available materials'}</h3>
    {result ? <><SearchStatus result={result} resources={resources} id={materialStatusId} />{result.indexRebuilt && <p role="status">The search index was rebuilt from saved materials.</p>}</> : <MaterialCount count={resources.length} page={currentPage} id={materialStatusId} />}
    <ul>{visible.map(item => <li key={item.id}><button aria-current={selected === item.id ? 'page' : undefined} onClick={() => open(item.id)}>{item.title}</button>{item.snippet && <p>{item.snippet}</p>}{'references' in item && item.references?.map((ref, index) => <p key={`${ref.sourceId}-${index}`}>Original {ref.sourceId}, {ref.locator.kind} {ref.locator.value}{!ref.originalAvailable && ' — original unavailable or corrupt'}</p>)}</li>)}</ul>
    {result ? <ContinueSearch search={search} beforeSearch={() => { focus.current = true; }} /> : <div className="actions"><button disabled={!currentPage} onClick={() => { focus.current = true; setPage(0); onContext({ page: 0 }); }}>First materials</button><button disabled={!currentPage} onClick={() => { focus.current = true; setPage(currentPage - 1); onContext({ page: currentPage - 1 }); }}>Previous materials</button><button disabled={currentPage >= lastPage} onClick={() => { focus.current = true; setPage(currentPage + 1); onContext({ page: currentPage + 1 }); }}>Next materials</button><button disabled={currentPage >= lastPage} onClick={() => { focus.current = true; setPage(lastPage); onContext({ page: lastPage }); }}>Last materials</button></div>}
  </section>;
}
function materialResults(result: SearchPage | null, resources: Entity[], page: number) {
  if (result) return result.items.map(item => ({ id: item.resourceId, title: item.title, snippet: item.snippet, references: item.references }));
  return resources.slice(page * 20, page * 20 + 20).map(item => ({ ...item, snippet: '' }));
}
function SearchForm({ search, beforeSearch, beforeEdit }: { search: MaterialSearch; beforeSearch: () => void; beforeEdit: () => void }) {
  const helpId = useId();
  const valid = search.query.length > 0 && [...search.query].length <= 256;
  function submit(event: FormEvent) { event.preventDefault(); if (valid) { beforeSearch(); void search.search(); } }
  return <><form onSubmit={submit}><label>Search titles, tags and text<input type="search" value={search.query} onFocus={beforeEdit} onChange={event => { beforeEdit(); search.edit(event.target.value); }} aria-describedby={helpId} /></label><p id={helpId}>Literal text, up to 256 characters. Searches saved titles, tags and canonical text in bounded batches.</p><button disabled={search.busy || !valid}>Search materials</button><button type="button" onClick={() => { beforeEdit(); search.edit(''); }}>Show all materials</button></form>{[...search.query].length > 256 && <p role="alert">Use at most 256 characters.</p>}{search.error && <p role="alert">{search.error}</p>}{search.busy && <p role="status">Searching saved materials…{search.scannedBatches > 0 && ` ${search.scannedBatches} batches checked.`}</p>}</>;
}
function ContinueSearch({ search, beforeSearch }: { search: MaterialSearch; beforeSearch: () => void }) {
  const offset = search.result?.nextOffset ?? null;
  return <button disabled={search.busy || offset === null} onClick={() => { if (offset !== null) { beforeSearch(); void search.search(offset); } }}>Continue search</button>;
}
function MaterialCount({ count, page, id }: { count: number; page: number; id: string }) {
  return <p id={id} role="status">{count ? `${count} materials. Showing ${page * 20 + 1}–${Math.min((page + 1) * 20, count)}.` : 'No active materials. Create a material to begin.'}</p>;
}
function SearchStatus({ result, resources, id }: { result: SearchPage; resources: Entity[]; id: string }) {
  const positions = result.items.map(item => resources.findIndex(resource => resource.id === item.resourceId) + 1).filter(position => position > 0);
  const range = positions.length ? `Matching material positions ${Math.min(...positions)}–${Math.max(...positions)} of ${resources.length}. ` : '';
  const matches = result.items.length ? `${result.items.length} matches in this batch.` : 'No matches in this batch.';
  return <p id={id} role="status">{range}{matches}{result.nextOffset !== null ? ' More materials remain to search.' : ' Search complete.'}</p>;
}
function ArchivePanel({ data, perform }: { data: Library; perform: Perform }) {
  const [open, setOpen] = useState(false);
  const groups = [['subject', data.subjects], ['goal', data.goals], ['collection', data.collections], ['resource', data.resources], ['unit', data.units], ['path', data.paths]] as const;
  return <details onToggle={event => setOpen(event.currentTarget.open)}><summary>Manage archived items</summary>{open && <><p>Archiving preserves the item and its identity. Restore an item to make it available again.</p>{groups.map(([kind, items]) => <ArchiveGroup key={kind} kind={kind} items={items} perform={perform} />)}<ul>{Object.values(data.placements).filter(item => item.archived).map(item => <li key={item.id}>Removed placement in {data.collections[item.collectionId]?.title}<button aria-label={`Restore placement ${item.id}`} onClick={() => void perform({ type: 'archive', target: { kind: 'placement', id: item.id }, archived: false })}>Restore placement</button></li>)}</ul></>}</details>;
}
function ArchiveGroup({ kind, items, perform }: { kind: 'subject' | 'goal' | 'collection' | 'resource' | 'unit' | 'path'; items: Record<string, Entity>; perform: Perform }) {
  const label = kind === 'resource' ? 'material' : kind;
  const [page, setPage] = useState(0); const [query, setQuery] = useState(''); const [order, setOrder] = useState('default');
  const heading = useRef<HTMLHeadingElement>(null); const focus = useRef(false);
  const collator = useMemo(() => new Intl.Collator('en', { sensitivity: 'base', numeric: true }), []);
  const all = Object.values(items); const entries = all.filter(item => item.title.toLowerCase().includes(query.toLowerCase()));
  if (order !== 'default') entries.sort((a, b) => (order === 'descending' ? -1 : 1) * (collator.compare(a.title, b.title) || a.id.localeCompare(b.id)));
  const lastPage = Math.max(0, Math.ceil(entries.length / 20) - 1); const currentPage = Math.min(page, lastPage);
  useEffect(() => { if (focus.current) { heading.current?.focus(); focus.current = false; } }, [currentPage]);
  function turn(next: number) { focus.current = true; setPage(next); }
  const title = `${label.charAt(0).toUpperCase() + label.slice(1)} archive controls`;
  return <section aria-label={title}><h3 ref={heading} tabIndex={-1}>{title}</h3>
    <label>Filter {label} archive items<input type="search" value={query} onChange={event => { focus.current = false; setQuery(event.target.value); setPage(0); }} /></label>
    <label>Sort {label} archive items<select value={order} onChange={event => { focus.current = false; setOrder(event.target.value); setPage(0); }}><option value="default">Default order</option><option value="ascending">Title ascending</option><option value="descending">Title descending</option></select></label>
    <p role="status">{query ? `Filter matches: ${entries.length} of ${all.length} ${label} items.` : `${entries.length} ${label} items.`}{entries.length ? ` Showing ${currentPage * 20 + 1}–${Math.min((currentPage + 1) * 20, entries.length)}.` : ' No items to show.'}</p>
    <ul>{entries.slice(currentPage * 20, currentPage * 20 + 20).map(item => <li key={item.id}>{item.title}{item.archived ? ' (archived)' : ''}<button type="button" aria-label={`${item.archived ? 'Restore' : 'Archive'} ${label} ${item.title}`} onClick={() => void perform({ type: 'archive', target: { kind, id: item.id }, archived: !item.archived })}>{item.archived ? 'Restore' : 'Archive'}</button></li>)}</ul>
    <div className="actions"><button type="button" disabled={!currentPage} onClick={() => turn(0)}>First {label} archive page</button><button type="button" disabled={!currentPage} onClick={() => turn(currentPage - 1)}>Previous {label} archive page</button><button type="button" disabled={currentPage >= lastPage} onClick={() => turn(currentPage + 1)}>Next {label} archive page</button><button type="button" disabled={currentPage >= lastPage} onClick={() => turn(lastPage)}>Last {label} archive page</button></div>
  </section>;
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
      <EntitySelect label="Reuse material" placeholder="Choose material" items={data.resources} value={resource} onChange={setResource} />
      <button disabled={!resource} onClick={() => void perform({ type: 'place', id: crypto.randomUUID(), collectionId: current.id, target: { kind: 'resource', id: resource } })}>Add shared placement</button>
      <EntitySelect label="Place learning unit" placeholder="Choose unit" items={data.units} value={unit} onChange={setUnit} />
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
