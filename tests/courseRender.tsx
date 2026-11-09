import { render as renderTree, type RenderOptions } from '@testing-library/react';
import type { ReactNode } from 'react';
import { ApplicationProvider } from '../src/application/ApplicationProvider';
import { createNativeCoursePorts } from '../src/courses/nativeCoursePort';
const ports = { ...createNativeCoursePorts('test-course'), lessons: undefined };
export function render(ui: ReactNode, options?: RenderOptions) {
  return renderTree(ui, { wrapper: ({ children }) => <ApplicationProvider ports={ports}>{children}</ApplicationProvider>, ...options });
}
