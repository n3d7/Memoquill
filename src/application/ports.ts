import type { Annotation, ApplyContent, Envelope, Library, ReadingPosition, SaveDocument, SavedDocument, SearchPage } from './contentTypes';
import type { OriginalSource, SaveSource, SelectedSource, SourceList, SourcePreview } from './sourceTypes';
import type { PreferencesSnapshot, SavePreferences, WorkspaceStatus } from './workspaceTypes';

/** Application contracts contain no IPC, UI state, or caller-supplied authority. */
export interface ContentPort {
  library(): Promise<Library>;
  searchResources(query: string, offset?: number): Promise<SearchPage>;
  applyContent(request: ApplyContent): Promise<Library>;
  readDocument(resourceId: string, revisionId?: string): Promise<SavedDocument | null>;
  saveDocument(request: SaveDocument): Promise<SavedDocument>;
  readingPosition(resourceId: string): Promise<ReadingPosition | null>;
  saveReadingPosition(request: Envelope & { position: ReadingPosition }): Promise<ReadingPosition>;
  annotations(resourceId: string): Promise<Annotation[]>;
  saveAnnotation(request: Envelope & { annotation: Annotation }): Promise<Annotation>;
}
export interface SourcePort {
  pickSource(request: Omit<Envelope, 'expectedRevision'>): Promise<SelectedSource | null>;
  sources(resourceId: string): Promise<SourceList>;
  saveSource(request: SaveSource): Promise<OriginalSource>;
  sourcePreview(sourceId: string): Promise<SourcePreview>;
}
export interface WorkspacePort {
  workspaceStatus(): Promise<WorkspaceStatus>;
  reopenWorkspace(): Promise<WorkspaceStatus>;
  saveReadingPreferences(request: SavePreferences): Promise<PreferencesSnapshot>;
}
