import { createContext, useContext, type ReactNode } from 'react';
import * as content from '../contentBridge';
import * as sources from '../sourceBridge';
import * as workspace from '../workspaceBridge';
import type { LessonPort } from '../lessons/types';
import { localGuestIdentity, type IdentityPort } from './identity';
import type { ContentPort, SourcePort, WorkspacePort } from './ports';

export interface ApplicationPorts {
  content: ContentPort;
  sources: SourcePort;
  workspace: WorkspacePort;
  identity: IdentityPort;
  lessons?: LessonPort;
}
// Defaults keep existing bridge consumers compatible; providers replace services per tree.
const defaults: ApplicationPorts = { content, sources, workspace, identity: localGuestIdentity };
const ApplicationContext = createContext<ApplicationPorts>(defaults);
export function ApplicationProvider({ ports, children }: { ports: Partial<ApplicationPorts>; children: ReactNode }) {
  const parent = useContext(ApplicationContext);
  return <ApplicationContext.Provider value={{ ...parent, ...ports }}>{children}</ApplicationContext.Provider>;
}
export const useContentPort = () => useContext(ApplicationContext).content;
export const useSourcePort = () => useContext(ApplicationContext).sources;
export const useWorkspacePort = () => useContext(ApplicationContext).workspace;
export const useIdentityPort = () => useContext(ApplicationContext).identity;
export function useLessonPort(): LessonPort {
  const port = useContext(ApplicationContext).lessons;
  if (!port) throw new Error('A lesson application port must be supplied by the application root');
  return port;
}
