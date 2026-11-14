import { invoke } from '@tauri-apps/api/core';
import type { ReviewPort } from './reviewTypes';
export function createNativeReviewPort(courseId: string): ReviewPort {
  const body = (args: object) => new TextEncoder().encode(JSON.stringify({ courseId, ...args }));
  return { overview: () => invoke('review_overview', body({})), apply: request => invoke('review_apply', body({ request })), recover: () => invoke('reopen_workspace', { courseId }) };
}
