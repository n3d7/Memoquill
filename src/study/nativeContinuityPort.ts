import { invoke } from '@tauri-apps/api/core';
import type { ContinuityPort } from './continuityTypes';

export function createNativeContinuityPort(courseId: string): ContinuityPort {
  return { sessions: (offset = 0) => invoke('study_resume_sessions', new TextEncoder().encode(JSON.stringify({ courseId, offset }))) };
}
