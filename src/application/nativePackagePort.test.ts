import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativePackagePort } from './nativePackagePort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

it('defaults history export to false and forwards explicit consent with the captured course', async () => {
  vi.mocked(invoke).mockReset();
  const port = createNativePackagePort('course-a');
  await port.exportPackage('portableContent', false);
  await port.exportPackage('portableContent', true, true);
  expect(vi.mocked(invoke).mock.calls).toEqual([
    ['package_export', { courseId: 'course-a', request: { kind: 'portableContent', includeOriginals: false, includeHistory: false } }],
    ['package_export', { courseId: 'course-a', request: { kind: 'portableContent', includeOriginals: true, includeHistory: true } }],
  ]);
});
