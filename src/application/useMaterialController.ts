import { useEffect, useState } from 'react';
import { useResourceDocument } from '../useResourceDocument';

/** Per mounted material: draft, resume and navigation protection never live in a shared store. */
export function useMaterialController(workspaceId: string, resourceId: string, onDirty: (dirty: boolean) => void) {
  const material = useResourceDocument(workspaceId, resourceId);
  const [resume, setResume] = useState<{ blockId: string; offsetUtf16: number }>();
  const [sourcePending, setSourcePending] = useState(false);
  const [contextPending, setContextPending] = useState(false);
  useEffect(() => { onDirty(material.dirty || material.hasPending || sourcePending || contextPending); return () => onDirty(false); }, [material.dirty, material.hasPending, sourcePending, contextPending, onDirty]);
  return { material, resume, setResume, contextPending, setContextPending, setSourcePending };
}
