import type { ActivityRevision } from '../lessons/types';

export type StudyMode = 'practice' | 'assessment';
export type DeclaredAssistance = 'unknown' | 'none' | 'present';
export interface StudyHelp { hintsRevealed: number; solutionRevealed: boolean; external: DeclaredAssistance }
export type StudyResponse = { kind: 'text'; value: string } | { kind: 'choice'; optionId: string } | { kind: 'procedure'; completedStepIds: string[]; observations: string };
export interface StudyDraft { response: StudyResponse | null; help: StudyHelp; firstSavedAtUnixMs?: number | null }
export interface StudyItem { bindingId: string; activity: ActivityRevision }
export interface StudySession { contractVersion: 1; id: string; unitId: string; compositionRevisionId: string; mode: StudyMode; assessmentId?: string | null; bindingIds?: string[] | null; phase?: StudyPhase; startedAtUnixMs?: number | null; revision: number; cursor: number; items: StudyItem[]; draft: StudyDraft }
export type AssessmentOutcome = 'correct' | 'incorrect' | 'unknown' | 'disputed';
export type StudyPhase = 'unspecified' | 'independent' | 'guided' | 'workedExample' | 'completion' | 'preInstruction' | 'delayedCheck' | 'changedContext';
export type AssessorIdentity = { kind: 'local' | 'learner' } | { kind: 'model'; provider: string; model: string; modelVersion: string; promptVersion: string; rubricVersion: string };
export interface JudgmentProvenance { activityRevisionId: string; compositionRevisionId: string; definitionSha256: string; assessor: AssessorIdentity }
export type JudgmentUncertainty = { kind: 'notEstimated' | 'notApplicable' } | { kind: 'reported'; confidenceBasisPoints: number; note: string };
export interface StudyAttempt { id: string; sessionId: string; itemIndex: number; activityRevisionId: string; response: StudyResponse; help: StudyHelp; selfRating: number | null; recallRating?: number | null; firstSavedAtUnixMs?: number | null; submittedAtUnixMs?: number | null }
export interface AssessmentJudgment { id: string; attemptId: string; source: 'deterministic' | 'selfReport' | 'model'; outcome: AssessmentOutcome; assessorVersion: string; supersedes: string | null; provenance?: JudgmentProvenance | null; uncertainty?: JudgmentUncertainty; recordedAtUnixMs?: number | null }
export interface StudyView { session: StudySession; attempts: StudyAttempt[]; judgments: AssessmentJudgment[]; changedContent: boolean; feedbackAvailable?: boolean }
export type StudyAction =
  | { kind: 'startScopedAssessment'; assessmentId: string; partIndex: number }
  | { kind: 'start'; unitId: string; compositionRevisionId: string; mode: StudyMode; phase?: StudyPhase; bindingIds?: string[] }
  | { kind: 'saveResponse'; response: StudyResponse; externalAssistance: DeclaredAssistance }
  | { kind: 'revealHint' | 'revealSolution' }
  | { kind: 'submit'; selfRating: number | null; recallRating?: number | null; selfOutcome: AssessmentOutcome | null }
  | { kind: 'correctSelfJudgment'; attemptId: string; supersedes: string; outcome: AssessmentOutcome };
export interface StudyMutation { operationId: string; sessionId: string; expectedRevision: number; action: StudyAction }
export interface StudyPort {
  sessions(unitId: string, offset?: number): Promise<StudySession[]>;
  read(sessionId: string): Promise<StudyView>;
  apply(request: StudyMutation): Promise<StudySession>;
  recover(): Promise<unknown>;
}
