import { createContext, useContext, type ReactNode } from 'react';
import * as content from '../contentBridge';
import * as sources from '../sourceBridge';
import * as workspace from '../workspaceBridge';
import type { LessonPort } from '../lessons/types';
import { localGuestIdentity, type IdentityPort } from './identity';
import type { ContentPort, SourcePort, WorkspacePort } from './ports';
import type { IngestionPort } from './ingestionTypes';

export interface ApplicationPorts {
  content: ContentPort;
  sources: SourcePort;
  workspace: WorkspacePort;
  identity: IdentityPort;
  lessons?: LessonPort;
  study?: import('../study/types').StudyPort;
  review?: import('../study/reviewTypes').ReviewPort;
  assessments?: import('../study/assessmentTypes').AssessmentPort;
  ingestion?: IngestionPort;
  packages?: import('./packageTypes').PackagePort;
}
// Defaults keep existing bridge consumers compatible; providers replace services per tree.
const defaults: ApplicationPorts = { content, sources, workspace, identity: localGuestIdentity };
const ActivityContext = createContext(true);
export const useApplicationActive = () => useContext(ActivityContext);
const ApplicationContext = createContext<ApplicationPorts>(defaults);
export function ApplicationProvider({ ports, children, active = true }: { ports: Partial<ApplicationPorts>; children: ReactNode; active?: boolean }) {
  const parent = useContext(ApplicationContext);
  return <ActivityContext.Provider value={active}><ApplicationContext.Provider value={{ ...parent, ...ports }}>{children}</ApplicationContext.Provider></ActivityContext.Provider>;
}
export const useContentPort = () => useContext(ApplicationContext).content;
export const useSourcePort = () => useContext(ApplicationContext).sources;
export const useWorkspacePort = () => useContext(ApplicationContext).workspace;
export const useIdentityPort = () => useContext(ApplicationContext).identity;
export const useOptionalLessonPort = () => useContext(ApplicationContext).lessons;
export const useOptionalAssessmentPort = () => useContext(ApplicationContext).assessments;
export const useOptionalReviewPort = () => useContext(ApplicationContext).review;
export const useOptionalStudyPort = () => useContext(ApplicationContext).study;
export const useOptionalIngestionPort = () => useContext(ApplicationContext).ingestion;
export function useLessonPort(): LessonPort {
  const port = useContext(ApplicationContext).lessons;
  if (!port) throw new Error('A lesson application port must be supplied by the application root');
  return port;
}
