import { invoke } from '@tauri-apps/api/core';
import type { Library, SearchPage, ApplyContent, SavedDocument, SaveDocument, ReadingPosition, Envelope, Annotation } from './contentTypes';
import type { ContentPort } from './ports';

export function createNativeContentPort(courseId: string): ContentPort {
  function library(): Promise<Library> { return invoke('library', { courseId }); }
  function searchResources(query: string, offset = 0): Promise<SearchPage> { return invoke('search_resources', { courseId, query, offset }); }
  function applyContent(request: ApplyContent): Promise<Library> { return invoke('apply_content', { courseId, request }); }
  function readDocument(resourceId: string, revisionId?: string): Promise<SavedDocument | null> {
    return invoke('read_document', { courseId, resourceId, revisionId: revisionId ?? null });
  }
  function saveDocument(request: SaveDocument): Promise<SavedDocument> { return invoke('save_document', { courseId, request }); }
  function readingPosition(resourceId: string): Promise<ReadingPosition | null> { return invoke('reading_position', { courseId, resourceId }); }
  function saveReadingPosition(request: Envelope & { position: ReadingPosition }): Promise<ReadingPosition> { return invoke('save_reading_position', { courseId, request }); }
  function annotations(resourceId: string): Promise<Annotation[]> { return invoke('annotations', { courseId, resourceId }); }
  function saveAnnotation(request: Envelope & { annotation: Annotation }): Promise<Annotation> { return invoke('save_annotation', { courseId, request }); }

  return { library, searchResources, applyContent, readDocument, saveDocument, readingPosition, saveReadingPosition, annotations, saveAnnotation };
}
export const library: ContentPort['library'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const searchResources: ContentPort['searchResources'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const applyContent: ContentPort['applyContent'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const readDocument: ContentPort['readDocument'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const saveDocument: ContentPort['saveDocument'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const readingPosition: ContentPort['readingPosition'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const saveReadingPosition: ContentPort['saveReadingPosition'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const annotations: ContentPort['annotations'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const saveAnnotation: ContentPort['saveAnnotation'] = async () => { throw new Error('A course-bound application provider is required.'); };
