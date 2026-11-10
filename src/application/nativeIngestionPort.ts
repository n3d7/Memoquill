import { invoke } from '@tauri-apps/api/core';
import type { IngestionPort } from './ingestionTypes';
export function createNativeIngestionPort(courseId: string): IngestionPort {
  return {
    create: request => invoke('ingestion_create', { courseId, request }),
    read: jobId => invoke('ingestion_read', { courseId, jobId }),
    list: (resourceId, offset) => invoke('ingestion_list', { courseId, resourceId, offset }),
    cancel: request => invoke('ingestion_cancel', { courseId, request }),
    apply: request => invoke('ingestion_apply', { courseId, request }),
    review: request => invoke('ingestion_review', { courseId, request }),
    reimportPreview: (jobId, previousJobId, mapping) => invoke('ingestion_reimport_preview', { courseId, jobId, previousJobId, mapping }),
    reimportApply: request => invoke('ingestion_reimport_apply', { courseId, request }),
    undoPreview: jobId => invoke('ingestion_undo_preview', { courseId, jobId }),
    undo: request => invoke('ingestion_undo', { courseId, request }),
  };
}
