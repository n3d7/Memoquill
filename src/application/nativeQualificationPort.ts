import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// Ephemeral M1 fixture contract. Persistent content is designed separately in M2.
export interface QualificationDraft {
  schemaVersion: 1;
  title: string;
  prose: string;
  formula: string;
  code: string;
}

interface DraftSummary {
  title: string;
  proseCharacters: number;
  codeLines: number;
}

export function checkDraft(draft: QualificationDraft): Promise<DraftSummary> {
  return invoke<DraftSummary>('check_draft', { draft });
}

export function pickQualificationSource(): Promise<string | null> {
  return open({ multiple: false, directory: false, title: 'Choose a source (name only; no import)' });
}
