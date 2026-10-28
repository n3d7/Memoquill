import { invoke } from '@tauri-apps/api/core';

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
