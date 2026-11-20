import { createContext, useContext, type ReactNode } from 'react';
import { builtins, type EffectiveSettings } from '../settings/types';

/** Frozen presentation and intentions, independent of evidence and assessment policy. */
export type StudyActivityFamily = 'all' | 'recall' | 'singleChoice' | 'constructed' | 'procedure';
export interface StudySettings extends EffectiveSettings { contractVersion: 1; itemLimit?: number; activityFamily?: StudyActivityFamily }
const Context = createContext<StudySettings | null>({ contractVersion: 1, ...builtins });
export function StudySettingsProvider({ settings, children }: { settings: EffectiveSettings | null; children: ReactNode }) {
  return <Context.Provider value={settings ? { contractVersion: 1, ...settings } : null}>{children}</Context.Provider>;
}
export function useStudySettings() { return useContext(Context); }
