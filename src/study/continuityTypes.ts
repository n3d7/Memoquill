import type { StudyMode, StudyPhase } from './types';

export interface ResumeSession {
  sessionId: string;
  unitId: string;
  title: string;
  compositionRevisionId: string;
  cursor: number;
  total: number;
  mode: StudyMode;
  phase: StudyPhase;
  savedResponse: boolean;
  changedContent: boolean;
}
export interface ResumePage { contractVersion: 1; offset: number; nextOffset: number | null; sessions: ResumeSession[] }
export interface ContinuityPort { sessions(offset?: number): Promise<ResumePage> }
