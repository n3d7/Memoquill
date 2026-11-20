export type Theme = 'system' | 'light' | 'dark';
export type TextScale = 100 | 125 | 150 | 200;
export interface NavigationContext {
  contractVersion: 1; subjectId: string; collectionId: string | null; resourceId: string;
  query: string; page: number; searchOffset: number; searchActive: boolean;
}
export interface ReadingPreferences { theme: Theme; textScale: TextScale; navigation?: NavigationContext | null; customization?: import('../settings/types').SettingsLayer | null; subjectSettings?: Record<string, import('../settings/types').SettingsLayer> | null; shortcuts?: import('../settings/shortcuts').ShortcutPreferences | null }
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
