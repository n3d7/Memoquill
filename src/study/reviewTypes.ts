export interface PracticalReminder { id: string; unitId: string; activityRevisionId: string; dueDay: number; intervalDays: number | null; archived: boolean; completedAtUnixMs: number | null }
export interface ReviewPlan { contractVersion: 1; revision: number; generation: number; timeZone: string; dayStartHour: number; dailyNewLimit: number; dailyReviewLimit: number; retentionBasisPoints: number; reminders: PracticalReminder[] }
export interface RecallQueueItem { unitId: string; compositionRevisionId: string; bindingId: string; activityRevisionId: string; prompt: string; dueDay: number | null; dueDate: string | null }
export interface ReviewOverview { plan: ReviewPlan; nowUnixMs: number; today: number; todayDate: string; timeZoneRules: string; clockRollback: boolean; newUsed: number; reviewsUsed: number; dueTotal: number; newTotal: number; offered: RecallQueueItem[] }
export type ReviewAction =
  | { kind: 'setPolicy'; timeZone: string; dayStartHour: number; dailyNewLimit: number; dailyReviewLimit: number; retentionBasisPoints: number }
  | { kind: 'addReminder'; id: string; unitId: string; activityRevisionId: string; dueDay: number; intervalDays: number | null }
  | { kind: 'completeReminder' | 'archiveReminder'; id: string };
export interface ReviewMutation { operationId: string; expectedRevision: number; action: ReviewAction }
export interface ReviewPort { overview(): Promise<ReviewOverview>; apply(request: ReviewMutation): Promise<ReviewPlan>; recover(): Promise<unknown> }
