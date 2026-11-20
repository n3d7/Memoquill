import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeProgressPort } from './nativeProgressPort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
it('binds each progress read to its originating course through awaits and paging', async () => {
  const a = createNativeProgressPort('course-a'); const b = createNativeProgressPort('course-b');
  await a.read('unit'); await b.read('unit', 32); await a.read('unit', 64);
  const calls = vi.mocked(invoke).mock.calls.map(([name, payload]) => {
    if (!ArrayBuffer.isView(payload)) throw new Error('Expected typed bytes');
    return [name, JSON.parse(new TextDecoder().decode(payload))];
  });
  expect(calls).toEqual([['study_progress', { courseId: 'course-a', unitId: 'unit', offset: 0 }], ['study_progress', { courseId: 'course-b', unitId: 'unit', offset: 32 }], ['study_progress', { courseId: 'course-a', unitId: 'unit', offset: 64 }]]);
});
