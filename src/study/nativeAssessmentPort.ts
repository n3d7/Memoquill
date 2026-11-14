import { invoke } from '@tauri-apps/api/core';
import type { AssessmentPort } from './assessmentTypes';
export function createNativeAssessmentPort(courseId: string): AssessmentPort {
  const body = (args: object) => new TextEncoder().encode(JSON.stringify({ courseId, ...args }));
  return { create: request => invoke('assessment_create', body({ request })), read: assessmentId => invoke('assessment_read', body({ assessmentId })), list: (offset = 0) => invoke('assessment_list', body({ offset })), recover: () => invoke('reopen_workspace', { courseId }) };
}
