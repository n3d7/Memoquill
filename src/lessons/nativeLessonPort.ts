import { invoke } from '@tauri-apps/api/core';
import type { LessonPort } from './types';

const body = (request: unknown) => new TextEncoder().encode(JSON.stringify({ request }));

/** Transport only: caller identity and authorization are constructed natively. */
export const nativeLessonPort: LessonPort = {
  discover: () => invoke('lesson_discover'),
  listDrafts: (unitId) => invoke('lesson_list_drafts', { unitId }),
  read: (unitId) => invoke('lesson_read', { unitId, revisionId: null }),
  readRevision: (unitId, revisionId) => invoke('lesson_read', { unitId, revisionId }),
  readSource: (unitId, revisionId) => invoke('lesson_read_source', { unitId, revisionId }),
  readResource: (unitId, revisionId) => invoke('lesson_read_resource', { unitId, revisionId }),
  readActivity: (unitId, revisionId) => invoke('lesson_read_activity', { unitId, revisionId }),
  createDraft: (request) => invoke('lesson_create_draft', body(request)),
  editDraft: (request) => invoke('lesson_edit_draft', body(request)),
  validateDraft: (request) => invoke('lesson_validate_draft', body(request)),
  previewDraft: (request) => invoke('lesson_preview_draft', body(request)),
  applyDraft: (request) => invoke('lesson_apply_draft', body(request)),
  discardDraft: (request) => invoke('lesson_discard_draft', body(request)),
  getReceipt: (operationId) => invoke('lesson_get_receipt', { operationId }),
};
