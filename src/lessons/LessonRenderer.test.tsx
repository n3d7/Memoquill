import { render, screen, within, fireEvent } from '@testing-library/react';
import { expect, it } from 'vitest';
import { LessonRenderer } from './LessonRenderer';
import { composition } from './testFixtures';
import type { Composition } from './types';
const exercise = { id: 'revision', activityId: 'exercise', unitId: 'english', definition: { family: 'recall' as const, version: 1 as const, prompt: 'Translate cat', answer: 'chat', hints: ['An animal'], evaluation: 'exact' as const } };
function lesson(unit: string): Composition { const c = composition(unit); c.bindings = [{ bindingId: 'a', targetKind: 'activity', targetId: 'exercise', revisionId: 'revision' }]; c.nodes[0].slots.body = ['first', 'second']; c.nodes.push(...['first', 'second'].map(nodeId => ({ nodeId, typeId: 'activity', typeVersion: 1, props: { bindingId: 'a', presentation: 'default' }, slots: {}, on: {} }))); return c; }
it('keeps repeated activity occurrences and reused node IDs independent across lessons; typing never submits', () => {
  render(<><section aria-label="English"><LessonRenderer composition={lesson('english')} activities={{ revision: exercise }} resources={{}} sources={{}} /></section><section aria-label="Physics"><LessonRenderer composition={lesson('physics')} activities={{ revision: exercise }} resources={{}} sources={{}} /></section></>);
  const english = within(screen.getByRole('region', { name: 'English' })); const physics = within(screen.getByRole('region', { name: 'Physics' }));
  fireEvent.change(english.getAllByRole('textbox', { name: 'Your response' })[0], { target: { value: 'chat' } });
  expect(english.getAllByRole('textbox')[1]).toHaveValue(''); expect(physics.getAllByRole('textbox')[0]).toHaveValue(''); expect(screen.queryByText('Exact answer matched.')).not.toBeInTheDocument();
  fireEvent.click(english.getAllByRole('button', { name: 'Check preview response' })[0]); expect(english.getByText('Exact answer matched.')).toBeInTheDocument(); expect(physics.queryByText('Exact answer matched.')).not.toBeInTheDocument();
});
it('preserves unknown nodes as inert placeholders', () => {
  const c = composition(); c.nodes[0].typeId = 'future'; c.nodes[0].props = { html: '<script>bad()</script>' };
  render(<LessonRenderer composition={c} activities={{}} resources={{}} sources={{}} />);
  expect(screen.getByText(/Unsupported component preserved/)).toBeInTheDocument(); expect(screen.queryByRole('button')).not.toBeInTheDocument();
});
