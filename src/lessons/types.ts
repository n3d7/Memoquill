import type { SavedDocument } from '../application/contentTypes';
import type { Document } from '../documentModel';
/** Offline composition contract. Revision IDs are immutable native-issued pins. */
export type Scalar = string | number | boolean | null;
export type ViewField = 'section.expanded' | 'response.present' | 'assistance.hint' | 'assistance.revealed' | 'preview.submitted';
export type Condition =
  | { kind: 'literal'; value: boolean }
  | { kind: 'eq'; field: ViewField; value: Scalar }
  | { kind: 'compare'; field: ViewField; op: 'lt' | 'lte' | 'gt' | 'gte'; value: number }
  | { kind: 'not'; expr: Condition }
  | { kind: 'all' | 'any'; exprs: Condition[] };
export type ActionId = 'view.toggleSection' | 'view.focusNode' | 'source.open' | 'response.update' | 'study.hint' | 'study.reveal' | 'study.submit' | 'study.next';
export interface ActionBinding { actionId: ActionId; version: 1; targetNodeId?: string; bindingId?: string; inputs: Record<string, Scalar | { eventField: 'value' }> }
export interface NodePropsMap {
  stack: { gap: 'compact' | 'normal' | 'wide'; headingLevel: 1 | 2 | 3 | 4 | 5 | 6 };
  columns: { ratio: 'equal' | 'widePrimary' | 'wideSecondary' };
  section: { title: string; collapsible: boolean };
  content: { bindingId: string; blockRange?: { startBlockId: string; endBlockId: string } };
  source: { bindingId: string; locator?: string };
  activity: { bindingId: string; presentation: 'default' | 'compact' };
  button: { label: string; style: 'primary' | 'secondary'; action: ActionBinding };
  status: { field: ViewField; label: string };
}
export type NodeType = keyof NodePropsMap;
export interface LessonNode { nodeId: string; typeId: string; typeVersion: number; props: NodePropsMap[NodeType] | Record<string, unknown>; slots: Record<string, string[]>; on: Record<string, ActionBinding>; visible?: Condition }
export interface Binding { bindingId: string; targetKind: 'resource' | 'activity' | 'source'; targetId: string; revisionId: string; locator?: string }
export interface Appearance { spacing: 'compact' | 'normal' | 'wide'; theme: 'system' | 'light' | 'dark'; typography: 'default' | 'serif'; width: 'normal' | 'wide'; emphasis: 'normal' | 'strong' }
export interface Composition { contractVersion: 1; unitId: string; rootNodeId: string; nodes: LessonNode[]; bindings: Binding[]; sourceBindingRefs: string[]; appearance: Appearance; authoringProvenance: { kind: 'manual' | 'assisted'; note?: string } }
export type ActivityDefinition =
  | { family: 'recall'; version: 1; prompt: string; answer: string; hints: string[]; evaluation: 'self' | 'exact' }
  | { family: 'singleChoice'; version: 1; prompt: string; options: { id: string; text: string }[]; correctOptionId: string; hints: string[]; evaluation: 'exact' }
  | { family: 'constructed'; version: 1; prompt: string; rubric: string; workedExampleBindingId?: string; hints: string[]; evaluation: 'self' }
  | { family: 'procedure'; version: 1; prompt: string; steps: { id: string; text: string }[]; reflection: string; hints: string[]; evaluation: 'self' };
export interface ActivityRevision { id: string; activityId: string; unitId: string; definition: ActivityDefinition }
export type ResourceDocument = Document;
export type JsonSchema = Record<string, unknown> | boolean;
export type LessonDiagnosticCode = 'invalid_props' | 'invalid_condition' | 'invalid_action' | 'invalid_reference' | 'invalid_tree' | 'invalid_operation' | 'unsupported_type' | 'budget_exceeded' | 'invalid_envelope' | 'revision_conflict';
export type LessonRepair = 'use_declared_value' | 'use_declared_field' | 'select_pinned_revision' | 'remove_or_replace_unsupported' | 'correct_tree' | 'split_request' | 'refresh_base' | 'use_closed_envelope';
export interface LessonDiagnostic { code: LessonDiagnosticCode; nodeId?: string; bindingId?: string; operationIndex?: number; field: string; repair: LessonRepair }
export interface LessonError { code: string; diagnostics?: LessonDiagnostic[] }
export interface LessonValidation { draftId: string; draftRevision: number; valid: boolean; diagnostics: LessonDiagnostic[] }
export interface ActionDescriptor { actionId: ActionId; version: 1; inputSchema: JsonSchema; resultSchema: JsonSchema; targetKinds: string[]; requiredCapabilities: string[]; validModes: string[]; availability: Record<string, 'available' | 'simulated' | 'capability_unavailable'>; sideEffectClass: string; idempotency: string; eventFieldSources: string[]; errorCodes: string[] }
export interface RegistryDescriptor { typeId: NodeType; typeVersion: 1; slots: string[]; events: string[]; props: Record<string, unknown>; propsSchema: JsonSchema; defaults: Record<string, unknown>; slotSchemas: Record<string, { minimum: number; maximum: number; ordered: boolean; allowedChildren: string }>; eventSchemas: Record<string, { payloadSchema: JsonSchema; allowedActions: ActionId[]; eventFields: Record<string, JsonSchema>; maximumActions: 1 }>; actions: ActionDescriptor[]; requiredCapabilities: string[]; validModes: string[]; state: { scope: string; schema: JsonSchema; maximumBytes: number; persistent: boolean }; accessibilityContract: string[]; fallback: { mode: 'inertPlaceholder'; preservesOpaquePayload: true; activationAllowed: false; diagnosticCode: 'unsupported_type'; repairs: LessonRepair[] }; supportedMigrations: number[]; schemas: Record<string, JsonSchema> }
export interface SourceBindingRevision { id: string; unitId: string; sourceId: string; assetDigest: string; filename: string; attribution: string | null; license: string | null; originUrl: string | null; locator?: string; declaredUsedRanges: string[]; extractionCoverage: 'notExtracted'; readCoverage: 'unknown'; warnings: string[] }
export type LessonOperation =
  | { kind: 'insertSubtree'; parent: string; slot: string; index: number; rootNodeId: string; nodes: LessonNode[]; bindings?: Binding[] }
  | { kind: 'replaceUnsupportedRoot'; node: LessonNode }
  | { kind: 'insertNode'; parent: string; slot: string; index: number; node: LessonNode }
  | { kind: 'removeNode'; id: string }
  | { kind: 'moveNode'; id: string; parent: string; slot: string; index: number }
  | { kind: 'configureNode'; id: string; props: NodePropsMap[NodeType] | Record<string, unknown> }
  | { kind: 'configureInteraction'; id: string; on: Record<string, ActionBinding>; visible?: Condition }
  | { kind: 'bindReference'; binding: Binding }
  | { kind: 'setAppearance'; tokens: Appearance }
  | { kind: 'createResourceRevision'; resourceId: string; expectedRevisionId: string | null; title: string; bindingId: string; document: ResourceDocument }
  | { kind: 'createSourceBindingRevision'; sourceId: string; bindingId: string; locator?: string; declaredUsedRanges: string[] }
  | { kind: 'createActivityRevision'; activityId: string; expectedRevisionId: string | null; bindingId: string; definition: ActivityDefinition };
export interface LessonDraft { id: string; unitId: string; revision: number; composition: Composition; baseHeadRevision: string | null }
export interface LessonPreview { draftId: string; draftRevision: number; composition: Composition; hash: string; warnings: string[] }
export interface SavedComposition { id: string; unitId: string; composition: Composition; parentRevisionId: string | null; registryContractVersion: 1; hash: string }
export interface LessonReceipt { operationId: string; saved: SavedComposition; actor: string; proposalHash: string; oldHead: string | null; newHead: string; createdRevisionIds: string[]; idMap: Record<string, string>; warnings: string[]; committed: true }
export interface LessonScope { contractVersion: 1; workspaceId: string; unitId: string }
export interface DraftScope extends LessonScope { draftId: string; expectedDraftRevision: number }
export interface CreateLessonDraft extends LessonScope { operationId: string; expectedHeadRevision: string | null }
export interface EditLessonDraft extends DraftScope { operationId: string; operations: LessonOperation[] }
export interface ApplyLessonDraft extends DraftScope { expectedHeadRevision: string | null; previewHash: string; operationId: string }
export interface DiscardLessonDraft extends DraftScope { operationId: string }
export interface LessonPort {
  discover(): Promise<RegistryDescriptor[]>;
  listDrafts(unitId: string): Promise<LessonDraft[]>;
  read(unitId: string): Promise<SavedComposition | null>;
  readRevision(unitId: string, revisionId: string): Promise<SavedComposition | null>;
  readSource(unitId: string, revisionId: string): Promise<SourceBindingRevision | null>;
  readResource(unitId: string, revisionId: string): Promise<SavedDocument | null>;
  readActivity(unitId: string, revisionId: string): Promise<ActivityRevision | null>;
  createDraft(request: CreateLessonDraft): Promise<LessonDraft>;
  editDraft(request: EditLessonDraft): Promise<LessonDraft>;
  validateDraft(request: DraftScope): Promise<LessonValidation>;
  previewDraft(request: DraftScope): Promise<LessonPreview>;
  applyDraft(request: ApplyLessonDraft): Promise<LessonReceipt>;
  discardDraft(request: DiscardLessonDraft): Promise<void>;
  getReceipt(operationId: string): Promise<LessonReceipt | null>;
}
