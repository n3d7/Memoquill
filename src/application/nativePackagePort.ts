import { invoke } from '@tauri-apps/api/core';
import type { PackagePort } from './packageTypes';
export function createNativePackagePort(courseId: string): PackagePort {
 return Object.freeze({
  exportPackage: (kind, includeOriginals, includeHistory = false) => invoke('package_export', { courseId, request: { kind, includeOriginals, includeHistory } }),
  inspect: () => invoke('package_inspect', { courseId }),
  confirm: request => invoke('package_confirm', { courseId, request }),
  discard: reviewId => invoke('package_discard', { courseId, reviewId }),
 } satisfies PackagePort);
}
