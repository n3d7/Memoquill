import { invoke } from '@tauri-apps/api/core';
import type { Envelope } from './contentBridge';

export interface SelectedSource { digest: string; byteLength: number; filename: string }
export interface NewSource {
  id: string; resourceId: string; digest: string; filename: string; mediaType: string;
  attribution: string | null; license: string | null; originUrl: string | null;
}
export interface OriginalSource extends NewSource { revision: number; byteLength: number }
export interface SourceList { revision: number; sources: OriginalSource[] }
export interface SourcePreview { source: OriginalSource; text: string | null; hex: string | null; truncated: boolean }
export type SaveSource = Envelope & { source: NewSource };
export function pickSource(request: Omit<Envelope, 'expectedRevision'>): Promise<SelectedSource | null> {
  return invoke('pick_source', { request });
}
export function sources(resourceId: string): Promise<SourceList> { return invoke('sources', { resourceId }); }
export function saveSource(request: SaveSource): Promise<OriginalSource> { return invoke('save_source', { request }); }
export function sourcePreview(sourceId: string): Promise<SourcePreview> { return invoke('source_preview', { sourceId }); }
