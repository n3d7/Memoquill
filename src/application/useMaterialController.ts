import { useEffect, useState } from 'react';
import { useResourceDocument } from '../useResourceDocument';
import { useSourceController } from './useSourceController';
import { useSavedContext } from './useSavedContext';

/** Per mounted material: draft, resume and navigation protection never live in a shared store. */
export function useMaterialController(workspaceId: string, resourceId: string, onDirty: (dirty: boolean) => void) {
  const material = useResourceDocument(workspaceId, resourceId);
  const [resume, setResume] = useState<{ blockId: string; offsetUtf16: number }>();
  const sources = useSourceController(workspaceId, resourceId);
  const context = useSavedContext(material.saved, workspaceId, setResume, material.busy || material.hasPending);
  const contextPending = context.dirty;
  const dirty = material.dirty || material.hasPending || sources.pending || contextPending;
  useEffect(() => { onDirty(dirty); return () => onDirty(false); }, [dirty, onDirty]);
  useEffect(() => {
    if (!dirty) return;
    function preventClose(event: BeforeUnloadEvent) { event.preventDefault(); }
    window.addEventListener('beforeunload', preventClose);
    return () => window.removeEventListener('beforeunload', preventClose);
  }, [dirty]);
  return { material, resume, setResume, contextPending, context, sources };
}
