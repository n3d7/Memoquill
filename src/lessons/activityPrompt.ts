import type { ActivityDefinition } from './types';
/** The cloze token is literal text, never a template or executable expression. */
export function activityPrompt(definition: ActivityDefinition): string {
  return definition.family === 'recall' && definition.version === 2
    ? definition.prompt.replace('{{answer}}', '[blank]') : definition.prompt;
}
