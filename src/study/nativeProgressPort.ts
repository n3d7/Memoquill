import { invoke } from '@tauri-apps/api/core';
import type { ProgressPort } from './progressTypes';

/** Course identity comes from the adapter, not a caller-supplied payload. */
export function createNativeProgressPort(courseId: string): ProgressPort {
  return { read: (unitId, offset = 0) => invoke('study_progress', new TextEncoder().encode(JSON.stringify({ courseId, unitId, offset }))) };
}
