import { invoke } from '@tauri-apps/api/core';
import type { Library, SearchPage, ApplyContent, SavedDocument, SaveDocument, ReadingPosition, Envelope, Annotation } from './contentTypes';
import type { ContentPort } from './ports';

export function library(): Promise<Library> { return invoke('library'); }
export function searchResources(query: string, offset = 0): Promise<SearchPage> { return invoke('search_resources', { query, offset }); }
export function applyContent(request: ApplyContent): Promise<Library> { return invoke('apply_content', { request }); }
export function readDocument(resourceId: string, revisionId?: string): Promise<SavedDocument | null> {
  return invoke('read_document', { resourceId, revisionId: revisionId ?? null });
}
export function saveDocument(request: SaveDocument): Promise<SavedDocument> { return invoke('save_document', { request }); }
export function readingPosition(resourceId: string): Promise<ReadingPosition | null> { return invoke('reading_position', { resourceId }); }
export function saveReadingPosition(request: Envelope & { position: ReadingPosition }): Promise<ReadingPosition> { return invoke('save_reading_position', { request }); }
export function annotations(resourceId: string): Promise<Annotation[]> { return invoke('annotations', { resourceId }); }
export function saveAnnotation(request: Envelope & { annotation: Annotation }): Promise<Annotation> { return invoke('save_annotation', { request }); }

export const nativeContentPort: ContentPort = { library, searchResources, applyContent, readDocument, saveDocument, readingPosition, saveReadingPosition, annotations, saveAnnotation };
