export type AssessmentScope = { kind: 'subject' | 'topic'; id: string };
export interface CreateAssessment { operationId: string; assessmentId: string; scope: AssessmentScope }
export interface AssessmentPin { bindingId: string; activityRevisionId: string; family: string }
export interface AssessmentPart { sessionId: string; unitId: string; unitTitle: string; objective: string; compositionRevisionId: string; items: AssessmentPin[] }
export interface FrozenAssessment { contractVersion: 1; id: string; scope: AssessmentScope; title: string; feedbackPolicy: 'afterAllPartsComplete'; createdAtUnixMs: number | null; parts: AssessmentPart[]; omissions: { unitId: string; bindingId: string | null; reason: string }[] }
export interface AssessmentSummary { id: string; title: string; partCount: number; itemCount: number }
export interface OutcomeCounts { deterministicCorrect: number; deterministicIncorrect: number; deterministicUnknown: number; selfCorrect: number; selfIncorrect: number; selfUnknown: number; selfDisputed: number }
export interface OutcomeCoverage { outcome: string; total: number; attempted: number; unattempted: number; declaredUnaided: number; externalUnknown: number; externalPresent: number; feedback: OutcomeCounts | null }
export interface AssessmentView { assessment: FrozenAssessment; parts: { sessionId: string; started: boolean; completed: number; total: number; changedContent: boolean }[]; feedbackAvailable: boolean; coverage: OutcomeCoverage[] }
export interface AssessmentPort { create(request: CreateAssessment): Promise<FrozenAssessment>; read(id: string): Promise<AssessmentView>; list(offset?: number): Promise<AssessmentSummary[]>; recover(): Promise<unknown> }
