import { beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeStudyPort } from './nativeStudyPort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeEach(() => vi.mocked(invoke).mockReset());

it('binds study mutations, reads, paging and recovery to the selected course', async () => {
  const port = createNativeStudyPort('course-a');
  const request = { operationId: 'operation', sessionId: 'session', expectedRevision: 1, action: { kind: 'revealHint' as const } };
  await port.apply(request); await port.read('session'); await port.sessions('unit', 64); await port.recover();
  const decode = (value: unknown) => { if (!ArrayBuffer.isView(value)) throw new Error('Expected a byte buffer'); return JSON.parse(new TextDecoder().decode(value)); };
  expect(decode(vi.mocked(invoke).mock.calls[0][1])).toEqual({ courseId: 'course-a', request });
  expect(decode(vi.mocked(invoke).mock.calls[1][1])).toEqual({ courseId: 'course-a', sessionId: 'session' });
  expect(decode(vi.mocked(invoke).mock.calls[2][1])).toEqual({ courseId: 'course-a', unitId: 'unit', offset: 64 });
  expect(vi.mocked(invoke).mock.calls[3]).toEqual(['reopen_workspace', { courseId: 'course-a' }]);
});
