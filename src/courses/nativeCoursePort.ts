import { createNativeAssessmentPort } from '../study/nativeAssessmentPort';
import { createNativeReviewPort } from '../study/nativeReviewPort';
import { createNativePackagePort } from '../application/nativePackagePort';
import { invoke } from '@tauri-apps/api/core';
import { createNativeContentPort } from '../application/nativeContentPort';
import { createNativeSourcePort } from '../application/nativeSourcePort';
import { createNativeWorkspacePort } from '../application/nativeWorkspacePort';
import { createNativeLessonPort } from '../lessons/nativeLessonPort';
import { createNativeStudyPort } from '../study/nativeStudyPort';
import { createNativeIngestionPort } from '../application/nativeIngestionPort';
import { localGuestIdentity } from '../application/identity';
import type { ApplicationPorts } from '../application/ApplicationProvider';
import type { CoursePort } from './types';
export const nativeCoursePort: CoursePort = {
  list: () => invoke('courses_list'),
  create: (operationId, title) => invoke('course_create', { request: { operationId, title } }),
  select: (courseId) => invoke('course_select', { courseId }),
};
/** Each closure retains its originating identity through awaits, retries and course switching. */
export function createNativeCoursePorts(courseId: string): ApplicationPorts {
  return Object.freeze({ assessments: createNativeAssessmentPort(courseId), review: createNativeReviewPort(courseId), packages: createNativePackagePort(courseId), content: createNativeContentPort(courseId), sources: createNativeSourcePort(courseId), workspace: createNativeWorkspacePort(courseId), lessons: createNativeLessonPort(courseId), study: createNativeStudyPort(courseId), ingestion: createNativeIngestionPort(courseId), identity: localGuestIdentity });
}
