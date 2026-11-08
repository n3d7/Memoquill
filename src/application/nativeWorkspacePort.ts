import { invoke } from '@tauri-apps/api/core';
import type { WorkspaceStatus, SavePreferences, PreferencesSnapshot } from './workspaceTypes';
import type { WorkspacePort } from './ports';

export function createNativeWorkspacePort(courseId: string): WorkspacePort {
  function workspaceStatus(): Promise<WorkspaceStatus> { return invoke('workspace_status', { courseId }); }
  function reopenWorkspace(): Promise<WorkspaceStatus> { return invoke('reopen_workspace', { courseId }); }
  function saveReadingPreferences(request: SavePreferences): Promise<PreferencesSnapshot> {
    return invoke('save_reading_preferences', { courseId, request });
  }

  return { workspaceStatus, reopenWorkspace, saveReadingPreferences };
}
export const workspaceStatus: WorkspacePort['workspaceStatus'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const reopenWorkspace: WorkspacePort['reopenWorkspace'] = async () => { throw new Error('A course-bound application provider is required.'); };
export const saveReadingPreferences: WorkspacePort['saveReadingPreferences'] = async () => { throw new Error('A course-bound application provider is required.'); };
