import { invoke } from '@tauri-apps/api/core';

export type Theme = 'system' | 'light' | 'dark';
export type TextScale = 100 | 125 | 150 | 200;
export interface ReadingPreferences { theme: Theme; textScale: TextScale }
export interface PreferencesSnapshot { workspaceId: string; revision: number; preferences: ReadingPreferences }
export interface SavePreferences {
  contractVersion: 1; workspaceId: string; operationId: string;
  expectedRevision: number; preferences: ReadingPreferences;
}
interface RecoveryReport {
  interruptedOperations: number; stagingFiles: number; orphanAssets: number;
  missingAssets: number; corruptAssets: number; unsafePaths: number;
}
export interface WorkspaceStatus {
  contractVersion: 1; schemaVersion: number; preferences: PreferencesSnapshot;
  recovery: RecoveryReport; sqliteVersion: string;
}
export function workspaceStatus(): Promise<WorkspaceStatus> { return invoke('workspace_status'); }
export function reopenWorkspace(): Promise<WorkspaceStatus> { return invoke('reopen_workspace'); }
export function saveReadingPreferences(request: SavePreferences): Promise<PreferencesSnapshot> {
  return invoke('save_reading_preferences', { request });
}
