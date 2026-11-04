import { invoke } from '@tauri-apps/api/core';
import type { LessonPort } from './types';

/** Transport only: caller identity and authorization are constructed natively. */
export const nativeLessonPort: LessonPort = {
  discover: () => invoke('lesson_discover'),
  listDrafts: (unitId) => invoke('lesson_list_drafts', { unitId }),
  read: (unitId) => invoke('lesson_read', { unitId, revisionId: null }),
  readRevision: (unitId, revisionId) => invoke('lesson_read', { unitId, revisionId }),
  readSource: (unitId, revisionId) => invoke('lesson_read_source', { unitId, revisionId }),
  readActivity: (unitId, revisionId) => invoke('lesson_read_activity', { unitId, revisionId }),
  createDraft: (request) => invoke('lesson_create_draft', { request }),
  editDraft: (request) => invoke('lesson_edit_draft', { request }),
  previewDraft: (request) => invoke('lesson_preview_draft', { request }),
  applyDraft: (request) => invoke('lesson_apply_draft', { request }),
  discardDraft: (request) => invoke('lesson_discard_draft', { request }),
  getReceipt: (operationId) => invoke('lesson_get_receipt', { operationId }),
};
