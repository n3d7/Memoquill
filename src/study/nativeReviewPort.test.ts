import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeReviewPort } from './nativeReviewPort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
it('keeps review requests and reconnects bound to the originating course', async () => {
  const first = createNativeReviewPort('first'); const second = createNativeReviewPort('second');
  await first.overview(); await second.overview();
  await first.apply({ operationId: 'op', expectedRevision: 2, action: { kind: 'completeReminder', id: 'reminder' } });
  await first.recover();
  const calls = vi.mocked(invoke).mock.calls;
  expect(JSON.parse(new TextDecoder().decode(calls[0][1] as Uint8Array))).toEqual({ courseId: 'first' });
  expect(JSON.parse(new TextDecoder().decode(calls[1][1] as Uint8Array))).toEqual({ courseId: 'second' });
  expect(JSON.parse(new TextDecoder().decode(calls[2][1] as Uint8Array))).toMatchObject({ courseId: 'first', request: { operationId: 'op', expectedRevision: 2 } });
  expect(calls[3]).toEqual(['reopen_workspace', { courseId: 'first' }]);
});
