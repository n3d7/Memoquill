import { invoke } from '@tauri-apps/api/core';
import type { Envelope } from './contentTypes';
import type { SelectedSource, SourceList, SaveSource, OriginalSource, SourcePreview } from './sourceTypes';
import type { SourcePort } from './ports';

export function pickSource(request: Omit<Envelope, 'expectedRevision'>): Promise<SelectedSource | null> {
  return invoke('pick_source', { request });
}
export function sources(resourceId: string): Promise<SourceList> { return invoke('sources', { resourceId }); }
export function saveSource(request: SaveSource): Promise<OriginalSource> { return invoke('save_source', { request }); }
export function sourcePreview(sourceId: string): Promise<SourcePreview> { return invoke('source_preview', { sourceId }); }

export const nativeSourcePort: SourcePort = { pickSource, sources, saveSource, sourcePreview };
