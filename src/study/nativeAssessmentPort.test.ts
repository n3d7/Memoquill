import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeAssessmentPort } from './nativeAssessmentPort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
it('binds frozen assessment mutations, reads and recovery to the originating course', async () => {
  const first = createNativeAssessmentPort('first'); const other = createNativeAssessmentPort('other');
  await first.create({ operationId: 'op', assessmentId: 'run', scope: { kind: 'topic', id: 'topic' } });
  await other.list(); await first.read('run'); await first.recover();
  const calls = vi.mocked(invoke).mock.calls;
  expect(JSON.parse(new TextDecoder().decode(calls[0][1] as Uint8Array))).toEqual({ courseId: 'first', request: { operationId: 'op', assessmentId: 'run', scope: { kind: 'topic', id: 'topic' } } });
  expect(JSON.parse(new TextDecoder().decode(calls[1][1] as Uint8Array))).toEqual({ courseId: 'other', offset: 0 });
  expect(JSON.parse(new TextDecoder().decode(calls[2][1] as Uint8Array))).toEqual({ courseId: 'first', assessmentId: 'run' });
  expect(calls[3]).toEqual(['reopen_workspace', { courseId: 'first' }]);
});
