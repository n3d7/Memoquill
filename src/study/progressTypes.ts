import type { AssessmentJudgment, AssessmentOutcome, StudyAttempt, StudyMode, StudyPhase } from './types';
import type { Binding } from '../lessons/types';

export interface ProgressCoverage {
  outcome: string;
  currentActivityBindings: number;
  evidencedCurrentActivityBindings: number;
  attempts: number;
  previousActivityAttempts: number;
  feedbackWithheldAttempts: number;
  unassessedAttempts: number;
  judgments: { source: AssessmentJudgment['source']; outcome: AssessmentOutcome; count: number }[];
}
export interface ProgressEvidence {
  attempt: StudyAttempt;
  bindingId: string;
  compositionRevisionId: string;
  outcome: string;
  mode: StudyMode;
  phase: StudyPhase;
  currentActivity: boolean;
  changedComposition: boolean;
  feedbackAvailable: boolean;
  judgments: AssessmentJudgment[];
  sources: Binding[];
}
export interface LessonProgress {
  contractVersion: 1;
  unitId: string;
  currentCompositionRevisionId: string | null;
  sessionCount: number;
  totalAttempts: number;
  offset: number;
  nextOffset: number | null;
  coverage: ProgressCoverage[];
  evidence: ProgressEvidence[];
}
export interface ProgressPort { read(unitId: string, offset?: number): Promise<LessonProgress> }
