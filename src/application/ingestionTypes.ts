import type { OriginalSource, SourceExtraction } from './sourceTypes';
type Envelope = { contractVersion: 1; workspaceId: string; operationId: string };
export type JsonPropertyMapping = { key: string | null; title: string | null; body: string | null };
export type Mapping = { keyColumn: number; titleColumn: number; bodyColumn: number; jsonProperties?: JsonPropertyMapping };
export type IngestionStatus = 'staged' | 'cancelled' | 'undone';
export interface Outcome { index: number; key: string; resourceId: string; documentRevisionId: string; beforeRevisionId?: string | null; beforeTitle?: string | null }
export type Disposition = 'pending' | 'omitted' | 'empty' | 'unresolved' | 'removedRetained';
export interface Decision { index: number | null; key: string | null; disposition: Disposition }
export interface IngestionJob { id: string; handlerVersion: number; revision: number; status: IngestionStatus; previousJobId: string | null; source: OriginalSource; extraction: SourceExtraction; mapping: Mapping | null; outcomes: Outcome[]; decisions?: Decision[] }
export interface JobSummary { id: string; sourceId: string; revision: number; status: IngestionStatus; recordCount: number; appliedCount: number; eligibleBaseline: boolean }
export interface CreateJob extends Envelope { sourceId: string; sourceRevision: number }
export interface JobAction extends Envelope { jobId: string; expectedRevision: number }
export interface ApplyJob extends JobAction { expectedLibraryRevision: number; mapping: Mapping; selected: number[] }
export interface ApplyReimport extends ApplyJob { previousJobId: string; expectedPreviousRevision: number; resolutions: { index: number; expectedDocumentRevisionId: string; expectedTitle: string }[] }
export interface UndoJob extends JobAction { expectedLibraryRevision: number }
export interface ReviewJob extends JobAction { previousJobId: string | null; expectedPreviousRevision: number | null; mapping: Mapping; decisions: Decision[] }
export type ReimportStatus = 'unchanged' | 'new' | 'changed' | 'conflict' | 'removed' | 'unresolved';
export interface ReimportValues { title: string; body: string; locator: IngestionJob['extraction']['records'][number]['locator'] }
export interface ReimportPreview { jobId: string; previousJobId: string; items: { index: number | null; key: string; resourceId: string | null; status: ReimportStatus; currentRevisionId: string | null; currentTitle: string | null; currentBody?: string | null; prior?: ReimportValues | null; incoming?: ReimportValues | null }[] }
export interface UndoPreview { jobId: string; canUndo: boolean; resources: string[]; diagnostics?: string[] }
export interface IngestionPort {
  create(request: CreateJob): Promise<IngestionJob>; read(jobId: string): Promise<IngestionJob>;
  list(resourceId: string, offset: number): Promise<JobSummary[]>; cancel(request: JobAction): Promise<IngestionJob>;
  apply(request: ApplyJob): Promise<IngestionJob>;
  review(request: ReviewJob): Promise<IngestionJob>;
  reimportPreview(jobId: string, previousJobId: string, mapping: Mapping): Promise<ReimportPreview>;
  reimportApply(request: ApplyReimport): Promise<IngestionJob>;
  undoPreview(jobId: string): Promise<UndoPreview>; undo(request: UndoJob): Promise<IngestionJob>;
}
