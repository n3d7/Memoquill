import { invoke } from '@tauri-apps/api/core';
import type { StudyPort } from './types';

/** Course identity is captured by this adapter; request payloads cannot replace it. */
export function createNativeStudyPort(courseId: string): StudyPort {
  const body = (args: object) => new TextEncoder().encode(JSON.stringify({ courseId, ...args }));
  return {
    sessions: (unitId, offset = 0) => invoke('study_sessions', body({ unitId, offset })),
    read: sessionId => invoke('study_read', body({ sessionId })),
    apply: request => invoke('study_apply', body({ request })),
    recover: () => invoke('reopen_workspace', { courseId }),
  };
}
