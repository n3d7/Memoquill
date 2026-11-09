export function isConfirmedRejection(error: unknown): boolean {
  return typeof error === 'object' && error !== null && 'code' in error &&
    ['revision_conflict', 'invalid_request', 'permission_denied', 'budget_exceeded'].includes(String(error.code));
}
export function failureMessage(error: unknown): string {
  const code = typeof error === 'object' && error !== null && 'code' in error ? String(error.code) : '';
  if (code === 'revision_conflict') return 'This item changed. Your draft is retained. Reload the saved version before trying again.';
  if (code === 'outcome_unconfirmed') return 'Save outcome is unconfirmed. Retry the same operation to check its durable receipt.';
  if (code === 'budget_exceeded') return 'The operation exceeds the supported limits. Select existing content or adjust the request.';
  if (code === 'invalid_request') return 'The change is invalid. Check the fields, references and supported content limits.';
  if (code === 'permission_denied') return 'The change is outside this workspace or available scope.';
  return 'The operation failed. Your edits are retained; you can retry.';
}
