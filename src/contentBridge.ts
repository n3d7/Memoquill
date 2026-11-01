import { invoke } from '@tauri-apps/api/core';
import type { Document } from './documentModel';

export interface Entity { id: string; title: string; archived: boolean }
export interface Subject extends Entity { priorKnowledge: string | null }
export interface Goal extends Entity { subjectId: string; criteria: string }
export interface Collection extends Entity { subjectId: string; parentId: string | null; position: number }
export type Target = { kind: 'resource' | 'unit'; id: string };
export interface Placement { id: string; collectionId: string; target: Target; position: number; archived: boolean }
export interface Unit extends Entity { objective: string; goalIds: string[]; resourceIds: string[] }
export interface Step { id: string; unitId: string }
export interface Path extends Entity { steps: Step[] }
export type RelationKind = 'prerequisite' | 'related' | 'reference';
export interface Relation { id: string; kind: RelationKind; from: string; to: string }
export interface Tag { id: string; title: string; resourceIds: string[] }
export interface Library {
  workspaceId: string; revision: number; subjects: Record<string, Subject>; goals: Record<string, Goal>;
  collections: Record<string, Collection>; resources: Record<string, Entity>; placements: Record<string, Placement>;
  units: Record<string, Unit>; paths: Record<string, Path>; relations: Record<string, Relation>; tags: Record<string, Tag>;
}
export type Command =
  | { type: 'createSubject' | 'updateSubject'; id: string; title: string; priorKnowledge: string | null }
  | { type: 'createGoal'; id: string; subjectId: string; title: string; criteria: string }
  | { type: 'createCollection' | 'moveCollection'; id: string; subjectId: string; parentId: string | null; title?: string }
  | { type: 'renameCollection' | 'renameResource' | 'renamePath' | 'createResource'; id: string; title: string }
  | { type: 'reorderCollections'; subjectId: string; parentId: string | null; orderedIds: string[] }
  | { type: 'createUnit' | 'updateUnit'; id: string; title: string; objective: string; goalIds: string[]; resourceIds: string[] }
  | { type: 'place'; id: string; collectionId: string; target: Target }
  | { type: 'movePlacement'; id: string; collectionId: string }
  | { type: 'reorderPlacements'; collectionId: string; orderedIds: string[] }
  | { type: 'createPath'; id: string; title: string; steps: Step[] }
  | { type: 'setPathSteps'; id: string; steps: Step[] }
  | { type: 'relate'; id: string; kind: RelationKind; from: string; to: string }
  | { type: 'removeRelation'; id: string }
  | { type: 'setTag'; id: string; title: string; resourceIds: string[] }
  | { type: 'archive'; target: { kind: 'subject' | 'goal' | 'collection' | 'resource' | 'placement' | 'unit' | 'path'; id: string }; archived: boolean };
export interface Envelope { contractVersion: 1; workspaceId: string; operationId: string; expectedRevision: number }
export interface ApplyContent extends Envelope { command: Command }
export interface SavedDocument { id: string; resourceId: string; revision: number; document: Document }
export interface SaveDocument extends Envelope { resourceId: string; document: Document }
export interface ReadingPosition { resourceId: string; revisionId: string; blockId: string; offsetUtf16: number; revision: number }
export interface Annotation { id: string; resourceId: string; revisionId: string; revision: number; body: string; anchor: { resourceId: string; revisionId: string; blockId: string; startUtf16: number; endUtf16: number; exactQuote: string; quoteHash: string | null } }
export function library(): Promise<Library> { return invoke('library'); }
export interface SearchPage { revision: number; items: Array<{ resourceId: string; title: string; snippet: string }>; nextOffset: number | null }
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
export function failureMessage(error: unknown): string {
  const code = typeof error === 'object' && error !== null && 'code' in error ? String(error.code) : '';
  if (code === 'revision_conflict') return 'This item changed. Your draft is retained. Reload the saved version before trying again.';
  if (code === 'outcome_unconfirmed') return 'Save outcome is unconfirmed. Retry the same operation to check its durable receipt.';
  if (code === 'invalid_request') return 'The change is invalid. Check the fields, references and supported content limits.';
  if (code === 'permission_denied') return 'The change is outside this workspace or available scope.';
  return 'The operation failed. Your edits are retained; you can retry.';
}
