import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeContinuityPort } from './nativeContinuityPort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
it('keeps resume queries and pagination bound to the originating course', async () => {
  const a = createNativeContinuityPort('course-a'); const b = createNativeContinuityPort('course-b');
  await a.sessions(); await b.sessions(64); await a.sessions(128);
  const calls = vi.mocked(invoke).mock.calls.map(([command, payload]) => {
    if (!ArrayBuffer.isView(payload)) throw new Error('Expected byte admission');
    return [command, JSON.parse(new TextDecoder().decode(payload))];
  });
  expect(calls).toEqual([['study_resume_sessions', { courseId: 'course-a', offset: 0 }], ['study_resume_sessions', { courseId: 'course-b', offset: 64 }], ['study_resume_sessions', { courseId: 'course-a', offset: 128 }]]);
});
