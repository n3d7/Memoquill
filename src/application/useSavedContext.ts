import { useEffect, useRef, useState } from 'react';
import { failureMessage, isConfirmedRejection } from './errors';
import type { Annotation, ReadingPosition, SavedDocument } from './contentTypes';
import type { ContentPort } from './ports';
import { useContentPort } from './ApplicationProvider';

type ContextRequest = { kind: 'position'; request: Parameters<ContentPort['saveReadingPosition']>[0] } | { kind: 'annotation'; request: Parameters<ContentPort['saveAnnotation']>[0] };
export function useSavedContext(saved: SavedDocument | null, workspaceId: string, onResume: (value: { blockId: string; offsetUtf16: number }) => void, locked: boolean) {
  const { readingPosition, annotations, saveReadingPosition, saveAnnotation } = useContentPort();
  const [blockId, setBlockId] = useState(saved?.document.blocks[0]?.id ?? '');
  const [position, setPosition] = useState<ReadingPosition | null>(null);
  const [notes, setNotes] = useState<Annotation[]>([]); const [body, setBody] = useState('');
  const [message, setMessage] = useState(''); const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const pending = useRef<ContextRequest | null>(null); const active = useRef(false);
  const loadGeneration = useRef(0);
  async function loadContext(live: () => boolean = () => true) {
    if (!saved || active.current || pending.current) return;
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
    let live = true;
    pending.current = null; active.current = false;
    setBlockId(saved?.document.blocks[0]?.id ?? ''); setPosition(null); setNotes([]); setBody(''); setMessage(''); setBusy(false); setLoading(saved !== null);
    void loadContext(() => live);
    return () => { live = false; loadGeneration.current++; };
  }, [workspaceId, saved?.id]);
  async function run(action: ContextRequest) {
    if (!saved || active.current || locked || loading) return;
    const generation = loadGeneration.current;
    active.current = true; pending.current = action; setBusy(true);
    try {
      if (action.kind === 'position') { const place = await saveReadingPosition(action.request); if (generation !== loadGeneration.current) return; setPosition(place); setMessage('Reading location saved locally.'); }
      else { const note = await saveAnnotation(action.request); if (generation !== loadGeneration.current) return; setNotes(list => [...list.filter(item => item.id !== note.id), note]); setBody(value => value === action.request.annotation.body ? '' : value); setMessage('Annotation saved on this exact revision.'); }
      pending.current = null;
    } catch (error) { if (generation === loadGeneration.current) { if (isConfirmedRejection(error)) pending.current = null; setMessage(failureMessage(error)); } }
    finally { if (generation === loadGeneration.current) { active.current = false; setBusy(false); } }
  }
  function remember() { if (!saved || pending.current || locked) return; return run({ kind: 'position', request: { contractVersion: 1, workspaceId, operationId: crypto.randomUUID(), expectedRevision: position?.revision ?? 0, position: { resourceId: saved.resourceId, revisionId: saved.id, blockId, offsetUtf16: 0, revision: position?.revision ?? 0 } } }); }
  function addNote() { if (!saved || pending.current || locked) return; return run({ kind: 'annotation', request: { contractVersion: 1, workspaceId, operationId: crypto.randomUUID(), expectedRevision: 0, annotation: { id: crypto.randomUUID(), resourceId: saved.resourceId, revisionId: saved.id, revision: 0, body, anchor: { resourceId: saved.resourceId, revisionId: saved.id, blockId, startUtf16: 0, endUtf16: 0, exactQuote: '', quoteHash: null } } } }); }
  return { saved, blockId, setBlockId, position, notes, body, setBody, message, busy, loading, locked,
    hasPending: pending.current !== null, dirty: body.length > 0 || pending.current !== null,
    remember, addNote, loadContext,
    retry: () => pending.current && run(pending.current),
    resume: () => saved && onResume({ blockId, offsetUtf16: position?.revisionId === saved.id && position.blockId === blockId ? position.offsetUtf16 : 0 }),
  };
}
export type SavedContextModel = ReturnType<typeof useSavedContext>;
