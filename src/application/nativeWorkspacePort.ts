import { invoke } from '@tauri-apps/api/core';
import type { WorkspaceStatus, SavePreferences, PreferencesSnapshot } from './workspaceTypes';
import type { WorkspacePort } from './ports';

export function workspaceStatus(): Promise<WorkspaceStatus> { return invoke('workspace_status'); }
export function reopenWorkspace(): Promise<WorkspaceStatus> { return invoke('reopen_workspace'); }
export function saveReadingPreferences(request: SavePreferences): Promise<PreferencesSnapshot> {
  return invoke('save_reading_preferences', { request });
}

export const nativeWorkspacePort: WorkspacePort = { workspaceStatus, reopenWorkspace, saveReadingPreferences };
