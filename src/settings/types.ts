import type { ReadingPreferences, Theme, TextScale } from '../application/workspaceTypes';
export interface SettingsOverrides { theme?: Theme; textScale?: TextScale; goal?: string; focus?: string }
export interface SettingsLayer { contractVersion: 1; overrides: SettingsOverrides }
export interface WorkspaceSettings { revision: number; settings: SettingsLayer }
export interface SaveWorkspaceSettings { operationId: string; expectedRevision: number; settings: SettingsLayer }
export interface SettingsPort { read(): Promise<WorkspaceSettings>; save(request: SaveWorkspaceSettings): Promise<WorkspaceSettings> }
export const emptyLayer = (): SettingsLayer => ({ contractVersion: 1, overrides: {} });
export const builtins = { theme: 'system' as Theme, textScale: 100 as TextScale, goal: '', focus: '' };
export type EffectiveSettings = typeof builtins;
export type SettingKey = keyof EffectiveSettings;
export const settingKeys: SettingKey[] = ['theme', 'textScale', 'goal', 'focus'];
export function courseLayer(preferences: ReadingPreferences): SettingsLayer {
  // Old workspaces retain their explicit appearance; adopting inheritance is deliberate.
  return preferences.customization ?? { contractVersion: 1, overrides: { theme: preferences.theme, textScale: preferences.textScale } };
}
export function resolveSettings(workspace: SettingsLayer, course: SettingsLayer, session: SettingsLayer, subject: SettingsLayer = emptyLayer()) {
  const effective = { ...builtins, ...workspace.overrides, ...course.overrides, ...subject.overrides, ...session.overrides };
  const sources = Object.fromEntries(settingKeys.map(key => [key, Object.hasOwn(session.overrides, key) ? 'session' : Object.hasOwn(subject.overrides, key) ? 'subject' : Object.hasOwn(course.overrides, key) ? 'course' : Object.hasOwn(workspace.overrides, key) ? 'workspace' : 'built-in'])) as Record<SettingKey, string>;
  return { effective, sources };
}
export function parseSettings(text: string): SettingsLayer {
  if (text.length > 32768) throw new Error('Settings JSON is too large.');
  const value: unknown = JSON.parse(text);
  const record = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);
  if (!record(value) || value.contractVersion !== 1 || Object.keys(value).some(k => !['contractVersion', 'overrides'].includes(k)) || !record(value.overrides)) throw new Error('Unsupported settings format.');
  const overrides: SettingsOverrides = {};
  for (const [key, item] of Object.entries(value.overrides)) {
    if (key === 'theme' && ['system', 'light', 'dark'].includes(String(item)) && typeof item === 'string') overrides.theme = item as Theme;
    else if (key === 'textScale' && typeof item === 'number' && [100, 125, 150, 200].includes(item)) overrides.textScale = item as TextScale;
    else if ((key === 'goal' || key === 'focus') && typeof item === 'string' && [...item].length <= 2048 && !/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f-\u009f]/.test(item)) overrides[key] = item;
    else throw new Error('Unknown or invalid setting.');
  }
  return { contractVersion: 1, overrides };
}
