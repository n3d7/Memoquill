import { useEffect } from 'react';
import { useApplicationActive } from '../application/ApplicationProvider';

export type ShortcutAction = 'openSettings' | 'focusSearch' | 'focusWorkspace';
export interface ShortcutPreferences { contractVersion: 1; bindings: Record<ShortcutAction, string | null> }
export const shortcutActions: ShortcutAction[] = ['openSettings', 'focusSearch', 'focusWorkspace'];
export const shortcutLabels: Record<ShortcutAction, string> = { openSettings: 'Settings shortcut', focusSearch: 'Search shortcut', focusWorkspace: 'Workspace shortcut' };
export const emptyShortcuts = (): ShortcutPreferences => ({ contractVersion: 1, bindings: { openSettings: null, focusSearch: null, focusWorkspace: null } });
export function parseShortcuts(text: string): ShortcutPreferences {
  if (text.length > 2048) throw new Error('Shortcut settings are too large.');
  const value: unknown = JSON.parse(text);
  const record = (v: unknown): v is Record<string, unknown> => !!v && typeof v === 'object' && !Array.isArray(v);
  if (!record(value) || value.contractVersion !== 1 || Object.keys(value).some(k => !['contractVersion', 'bindings'].includes(k)) || !record(value.bindings) || Object.keys(value.bindings).some(k => !shortcutActions.includes(k as ShortcutAction))) throw new Error('Unsupported shortcut settings.');
  const result = emptyShortcuts(); const used = new Set<string>();
  for (const action of shortcutActions) {
    const binding = value.bindings[action] ?? null;
    if (binding !== null && (typeof binding !== 'string' || !/^Ctrl\+Alt\+Shift\+[A-Z]$/.test(binding) || binding.endsWith('+R') || used.has(binding))) throw new Error('Use distinct Ctrl+Alt+Shift+letter bindings. R is reserved for desktop recording; other modifier patterns are unavailable.');
    if (typeof binding === 'string') used.add(binding);
    result.bindings[action] = binding as string | null;
  }
  return result;
}
export function useKeyboardShortcuts(preferences: ShortcutPreferences | null | undefined, actions: Record<ShortcutAction, () => boolean>) {
  const active = useApplicationActive();
  useEffect(() => {
    if (!active || !preferences) return;
    const handle = (event: KeyboardEvent) => {
      const target = event.target;
      if (event.defaultPrevented || event.isComposing || event.repeat || event.getModifierState('AltGraph') || !event.ctrlKey || !event.altKey || !event.shiftKey || event.metaKey || !/^Key[A-Z]$/.test(event.code) || target instanceof Element && target.closest('input,textarea,select,[contenteditable]:not([contenteditable="false"]),[role="textbox"]')) return;
      const binding = `Ctrl+Alt+Shift+${event.code.slice(3)}`;
      const action = shortcutActions.find(key => preferences.bindings[key] === binding);
      if (action && actions[action]()) { event.preventDefault(); event.stopPropagation(); }
    };
    document.addEventListener('keydown', handle);
    return () => document.removeEventListener('keydown', handle);
  }, [preferences, actions, active]);
}
