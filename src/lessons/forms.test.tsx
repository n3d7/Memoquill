import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { NodeEditor } from './NodeEditor';
import { defaults } from './LessonForms';
import { composition } from './testFixtures';
import type { ActivityDefinition, LessonNode, RegistryDescriptor, SourceBindingRevision } from './types';
import { ActivityFields } from './LessonForms';
import { useState } from 'react';
import type { ResolvedLesson } from './LessonRenderer';
const resolved: ResolvedLesson = { resources: {}, activities: {}, sources: {} };
it('edits canonical pinned material through a lesson resource revision with its exact base', async () => {
  const c = composition(); c.bindings = [{ bindingId: 'material', targetKind: 'resource', targetId: 'resource', revisionId: 'pinned' }];
  const document = { schemaVersion: 1 as const, blocks: [{ id: 'stable-block', content: { type: 'paragraph' as const, inlines: [{ text: 'Learning content', marks: [] }] } }] }; const node: LessonNode = { nodeId: 'content', typeId: 'content', typeVersion: 1, props: { bindingId: 'material' }, slots: {}, on: {} }; const save = vi.fn(async () => true);
  c.nodes.push(node); render(<NodeEditor node={node} composition={c} resolved={{ ...resolved, resources: { pinned: { id: 'pinned', resourceId: 'resource', revision: 1, document } } }} disabled={false} save={save} />);
  await screen.findByRole('textbox', { name: 'Lesson material 2' }); fireEvent.click(screen.getByRole('button', { name: 'Save material revision for this lesson' }));
  expect(save).toHaveBeenCalledWith([{ kind: 'createResourceRevision', resourceId: 'resource', expectedRevisionId: 'pinned', bindingId: 'material', title: 'Lesson material', document }]);
  fireEvent.click(screen.getByLabelText('Show selected block range')); fireEvent.click(screen.getByRole('button', { name: 'Save component settings' }));
  await waitFor(() => expect(save).toHaveBeenLastCalledWith([{ kind: 'configureNode', id: 'content', props: { bindingId: 'material', blockRange: { startBlockId: 'stable-block', endBlockId: 'stable-block' } } }]));
});
it('repairs an unsupported root explicitly while retaining its child node identities', () => {
  const c = composition(); c.nodes[0] = { ...c.nodes[0], typeId: 'future', slots: { opaque: ['child'] } }; const save = vi.fn(async () => true);
  render(<NodeEditor node={c.nodes[0]} composition={c} resolved={resolved} disabled={false} save={save} />); fireEvent.click(screen.getByRole('button', { name: 'Replace unsupported root with safe layout' }));
  expect(save).toHaveBeenCalledWith([{ kind: 'replaceUnsupportedRoot', node: expect.objectContaining({ typeId: 'stack', slots: { body: ['child'] } }) }]); expect(screen.queryByRole('button', { name: 'Save component settings' })).not.toBeInTheDocument();
});
it.each([['stack', 'Stack gap', 'wide', 'gap'], ['columns', 'Column widths', 'widePrimary', 'ratio'], ['section', 'Section title', 'Thermodynamics', 'title'], ['button', 'Button label', 'Help with this step', 'label'], ['status', 'Status label', 'Preview state', 'label'], ['activity', 'Exercise presentation', 'compact', 'presentation'], ['source', 'Original location', 'p. 12', 'locator']] as const)('configures %s with named typed fields', async (type, label, value, field) => {
  const c = composition(); const node: LessonNode = { nodeId: 'edit', typeId: type, typeVersion: 1, props: defaults(type), slots: {}, on: {} }; c.nodes.push(node); const save = vi.fn(async () => true);
  render(<NodeEditor node={node} composition={c} resolved={resolved} disabled={false} save={save} />); fireEvent.change(screen.getByLabelText(label), { target: { value } }); fireEvent.click(screen.getByRole('button', { name: 'Save component settings' }));
  await waitFor(() => expect(save).toHaveBeenCalledWith([{ kind: 'configureNode', id: 'edit', props: expect.objectContaining({ [field]: value }) }]));
});
it('authors bounded source locators and declared ranges without claiming extraction or reading coverage', async () => {
  const c = composition(); c.bindings = [{ bindingId: 'source-binding', targetKind: 'source', targetId: 'source', revisionId: 'pin' }];
  const source: SourceBindingRevision = { id: 'pin', unitId: 'english', sourceId: 'source', assetDigest: 'digest', filename: 'Book.pdf', attribution: null, license: null, originUrl: null, declaredUsedRanges: [], extractionCoverage: 'notExtracted', readCoverage: 'unknown', warnings: [] };
  const node = { nodeId: 'source-node', typeId: 'source', typeVersion: 1, props: { bindingId: 'source-binding' }, slots: {}, on: {} }; const save = vi.fn(async () => true);
  render(<NodeEditor node={node} composition={c} resolved={{ ...resolved, sources: { pin: source } }} disabled={false} save={save} />);
  fireEvent.change(await screen.findByLabelText('Pinned original locator'), { target: { value: 'Chapter 2' } }); fireEvent.change(screen.getByLabelText('Declared ranges (one per line)'), { target: { value: 'pages 12–14\nfigure 3' } }); fireEvent.click(screen.getByRole('button', { name: 'Save original snapshot for this lesson' }));
  expect(save).toHaveBeenCalledWith([{ kind: 'createSourceBindingRevision', sourceId: 'source', bindingId: 'source-binding', locator: 'Chapter 2', declaredUsedRanges: ['pages 12–14', 'figure 3'] }]);
});
it('configures a typed visibility condition and one named event action without raw handlers', () => {
  const c = composition(); const node: LessonNode = { nodeId: 'section', typeId: 'section', typeVersion: 1, props: defaults('section'), slots: { body: [] }, on: {} }; c.nodes.push(node); const save = vi.fn(async () => true);
  const registry = { events: ['toggle'], eventSchemas: { toggle: { allowedActions: ['view.toggleSection', 'view.focusNode'] } } } as unknown as RegistryDescriptor;
  render(<NodeEditor node={node} composition={c} registry={registry} resolved={resolved} disabled={false} save={save} />);
  fireEvent.change(screen.getByLabelText('Visibility condition type'), { target: { value: 'eq' } }); fireEvent.click(screen.getByLabelText('Enable toggle event')); fireEvent.click(screen.getByRole('button', { name: 'Save visibility and events' }));
  expect(save).toHaveBeenCalledWith([{ kind: 'configureInteraction', id: 'section', on: { toggle: { actionId: 'view.toggleSection', version: 1, targetNodeId: 'section', inputs: { expanded: true } } }, visible: { kind: 'eq', field: 'response.present', value: true } }]);
});
it.each(['recall', 'singleChoice', 'constructed', 'procedure'] as const)('authors the %s activity family with explicit evaluation', family => {
  function Form() { const [value, set] = useState<ActivityDefinition>({ family: 'recall', version: 1, prompt: 'Prompt', answer: 'Answer', hints: [], evaluation: 'self' }); return <ActivityFields definition={value} set={set} />; }
  render(<Form />); fireEvent.change(screen.getByLabelText('Exercise family'), { target: { value: family } }); expect(screen.getByLabelText('Exercise prompt')).toHaveValue('Prompt');
  expect(screen.getByLabelText(family === 'recall' ? 'Expected answer' : family === 'singleChoice' ? 'Option 1' : family === 'constructed' ? 'Assessment rubric' : 'Step 1')).toBeInTheDocument();
});
it('keeps a newline while typing a second hint in a controlled activity form', () => {
  function Form() { const [value, set] = useState<ActivityDefinition>({ family: 'recall', version: 1, prompt: 'Prompt', answer: 'Answer', hints: [], evaluation: 'self' }); return <ActivityFields definition={value} set={set} />; }
  render(<Form />); const hints = screen.getByLabelText('Hints (one per line)'); fireEvent.change(hints, { target: { value: 'First hint\n' } }); expect(hints).toHaveValue('First hint\n'); fireEvent.change(hints, { target: { value: 'First hint\nSecond hint' } }); expect(hints).toHaveValue('First hint\nSecond hint');
});
