import type { ActivityDefinition } from './types';
/** The cloze token is literal text, never a template or executable expression. */
export function activityPrompt(definition: ActivityDefinition): string {
  return definition.family === 'recall' && definition.version === 2
    ? definition.prompt.replace('{{answer}}', '[blank]') : definition.prompt;
}

/** Expected repair is reference material, not part of the initial flawed prompt. */
export function constructedReference(definition: Extract<ActivityDefinition, { family: 'constructed' }>): string {
  return definition.version === 3
    ? `Expected correction: ${definition.instruction.expectedCorrection}\nRubric: ${definition.rubric}`
    : definition.rubric;
}
