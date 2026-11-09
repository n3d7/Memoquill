import { invoke } from '@tauri-apps/api/core';
import type { Envelope } from './contentTypes';
import type { SelectedSource, SourceList, SaveSource, OriginalSource, SourcePreview } from './sourceTypes';
import type { SourcePort } from './ports';

export function createNativeSourcePort(courseId: string): SourcePort {
  function pickSource(request: Omit<Envelope, 'expectedRevision'>): Promise<SelectedSource | null> {
    return invoke('pick_source', { courseId, request });
  }
  function sources(resourceId: string): Promise<SourceList> { return invoke('sources', { courseId, resourceId }); }
  function saveSource(request: SaveSource): Promise<OriginalSource> { return invoke('save_source', { courseId, request }); }
  function sourcePreview(sourceId: string): Promise<SourcePreview> { return invoke('source_preview', { courseId, sourceId }); }

  return { pickSource, sources, saveSource, sourcePreview };
}
export const pickSource: SourcePort['pickSource'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const sources: SourcePort['sources'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const saveSource: SourcePort['saveSource'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const sourcePreview: SourcePort['sourcePreview'] = async () => { throw new Error('A course-bound application provider is required.'); };
