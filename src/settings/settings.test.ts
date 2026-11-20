import { expect, it } from 'vitest';
import { courseLayer, emptyLayer, parseSettings, resolveSettings } from './types';
it('resolves each setting independently, including explicit empty notes and legacy appearance', () => {
  const workspace = parseSettings('{"contractVersion":1,"overrides":{"theme":"dark","textScale":150,"goal":"Long-term goal","focus":"Workspace focus"}}');
  const course = parseSettings('{"contractVersion":1,"overrides":{"textScale":125,"goal":""}}');
  const session = parseSettings('{"contractVersion":1,"overrides":{"focus":"Temporary focus"}}');
  expect(resolveSettings(workspace, course, session)).toEqual({ effective: { theme: 'dark', textScale: 125, goal: '', focus: 'Temporary focus' }, sources: { theme: 'workspace', textScale: 'course', goal: 'course', focus: 'session' } });
  expect(resolveSettings(workspace, course, emptyLayer()).effective.focus).toBe('Workspace focus');
  expect(courseLayer({ theme: 'light', textScale: 200 }).overrides).toEqual({ theme: 'light', textScale: 200 });
  expect(courseLayer({ theme: 'light', textScale: 200, customization: emptyLayer() })).toEqual(emptyLayer());
  expect(JSON.parse(JSON.stringify(course))).toEqual(course);
});
it('rejects future versions, unknown/prototype fields, invalid types, control text and oversized input', () => {
  for (const value of [null, [], { contractVersion: 2, overrides: {} }, { contractVersion: 1, overrides: {}, extra: 1 }, { contractVersion: 1, overrides: { unexpected: true } }, { contractVersion: 1, overrides: { theme: null } }, { contractVersion: 1, overrides: { textScale: '125' } }, { contractVersion: 1, overrides: { textScale: 400 } }, { contractVersion: 1, overrides: { goal: 'x'.repeat(2049) } }, { contractVersion: 1, overrides: { focus: '\u0000' } }]) expect(() => parseSettings(JSON.stringify(value))).toThrow();
  expect(() => parseSettings('{"contractVersion":1,"overrides":{"__proto__":{"goal":"injected"}}}')).toThrow();
  expect(() => parseSettings(' '.repeat(32769))).toThrow();
  expect(parseSettings(JSON.stringify({ contractVersion: 1, overrides: { goal: '😀'.repeat(2048), focus: 'Line\n\tTwo' } })).overrides.goal).toHaveLength(4096);
});

it('applies subject overrides between course and temporary settings without mutating either', () => {
  const course = parseSettings('{"contractVersion":1,"overrides":{"theme":"light","goal":"Course goal"}}');
  const subject = parseSettings('{"contractVersion":1,"overrides":{"theme":"dark","focus":"Subject focus"}}');
  const session = parseSettings('{"contractVersion":1,"overrides":{"focus":"Temporary focus"}}');
  const result = resolveSettings(emptyLayer(), course, session, subject);
  expect(result.effective).toEqual({ theme: 'dark', textScale: 100, goal: 'Course goal', focus: 'Temporary focus' });
  expect(result.sources).toMatchObject({ theme: 'subject', goal: 'course', focus: 'session' });
  expect(subject.overrides.focus).toBe('Subject focus');
});
