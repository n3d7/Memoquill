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
it('reads one selected occurrence in the same lesson without coupling repeated activities or Units', () => {
  const make = (unitId: string) => { const c = lesson(unitId); c.nodes[0].slots.body.push('one-status', 'two-status'); c.nodes.push(...['first', 'second'].map((targetNodeId, i) => ({ nodeId: i === 0 ? 'one-status' : 'two-status', typeId: 'status', typeVersion: 1, props: { field: 'response.present', label: i === 0 ? 'First response present' : 'Second response present', targetNodeId }, slots: {}, on: {} }))); return c; };
  render(<><section aria-label="English statuses"><LessonRenderer composition={make('english')} activities={{ revision: exercise }} resources={{}} sources={{}} /></section><section aria-label="Physics statuses"><LessonRenderer composition={make('physics')} activities={{ revision: exercise }} resources={{}} sources={{}} /></section></>);
  const english = within(screen.getByRole('region', { name: 'English statuses' })); const physics = within(screen.getByRole('region', { name: 'Physics statuses' }));
  fireEvent.change(english.getAllByRole('textbox', { name: 'Your response' })[0], { target: { value: 'A response' } });
  expect(english.getByText('First response present: true')).toBeInTheDocument(); expect(english.getByText('Second response present: false')).toBeInTheDocument(); expect(physics.getByText('First response present: false')).toBeInTheDocument();
});
it('uses authored heading levels and reveals a pinned worked example only on an explicit preview action', () => {
  const c = composition(); c.nodes[0].props = { gap: 'normal', headingLevel: 3 }; c.nodes[0].slots.body = ['section']; c.nodes.push({ nodeId: 'section', typeId: 'section', typeVersion: 1, props: { title: 'Reasoning', collapsible: false }, slots: { body: ['exercise'] }, on: {} }, { nodeId: 'exercise', typeId: 'activity', typeVersion: 1, props: { bindingId: 'a', presentation: 'default' }, slots: {}, on: {} });
  c.bindings = [{ bindingId: 'a', targetKind: 'activity', targetId: 'a', revisionId: 'activity' }, { bindingId: 'example', targetKind: 'resource', targetId: 'resource', revisionId: 'resource' }];
  render(<LessonRenderer composition={c} activities={{ activity: { id: 'activity', activityId: 'a', unitId: 'english', definition: { family: 'constructed', version: 1, prompt: 'Explain your reasoning', rubric: 'Check each inference', workedExampleBindingId: 'example', hints: [], evaluation: 'self' } } }} resources={{ resource: { id: 'resource', resourceId: 'resource', revision: 1, document: { schemaVersion: 1, blocks: [{ id: 'p', content: { type: 'paragraph', inlines: [{ text: 'Worked solution', marks: [] }] } }] } } }} sources={{}} />);
  expect(screen.getByRole('heading', { name: 'Reasoning', level: 3 })).toBeInTheDocument(); expect(screen.queryByText('Worked solution')).not.toBeInTheDocument(); fireEvent.click(screen.getByRole('button', { name: 'Reveal preview guidance' })); expect(screen.getByText('Worked solution')).toBeInTheDocument();
});
