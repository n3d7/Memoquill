import { useState, type FormEvent } from 'react';
import type { ActionBinding, ActionId, ActivityDefinition, Composition, Condition, LessonNode, RegistryDescriptor, Scalar, ViewField } from './types';
import { viewFields } from './LessonRenderer';

export const nodeTypes = ['stack', 'columns', 'section', 'content', 'source', 'activity', 'button', 'status'] as const;
export function Select({ label, value, values, onChange, disabled = false }: { label: string; value: string; values: readonly string[]; onChange: (value: string) => void; disabled?: boolean }) { return <label>{label}<select value={value} disabled={disabled} onChange={e => onChange(e.target.value)}>{values.map(value => <option key={value} value={value}>{value || 'Choose'}</option>)}</select></label>; }
export function Text({ label, value, onChange, multiline = false }: { label: string; value: string; onChange: (value: string) => void; multiline?: boolean }) { return <label>{label}{multiline ? <textarea value={value} onChange={e => onChange(e.target.value)} /> : <input value={value} onChange={e => onChange(e.target.value)} />}</label>; }
export function Check({ label, value, onChange }: { label: string; value: boolean; onChange: (value: boolean) => void }) { return <label><input type="checkbox" checked={value} onChange={e => onChange(e.target.checked)} />{label}</label>; }
export function defaults(type: string): LessonNode['props'] {
  switch (type) {
    case 'stack': return { gap: 'normal', headingLevel: 2 };
    case 'columns': return { ratio: 'equal' };
    case 'section': return { title: 'New section', collapsible: true };
    case 'content': case 'source': return { bindingId: '' };
    case 'activity': return { bindingId: '', presentation: 'default' };
    case 'button': return { label: 'Show hint', style: 'secondary', action: { actionId: 'study.hint', version: 1, targetNodeId: '', inputs: {} } };
    case 'status': return { field: 'response.present', label: 'Response present' };
    default: return {};
  }
}
export function defaultActivity(family: ActivityDefinition['family']): ActivityDefinition {
  const common = { version: 1 as const, prompt: 'New exercise', hints: [] };
  switch (family) {
    case 'recall': return { ...common, family, answer: 'Answer', evaluation: 'self' };
    case 'singleChoice': return { ...common, family, options: [{ id: crypto.randomUUID(), text: 'First option' }, { id: crypto.randomUUID(), text: 'Second option' }], correctOptionId: '', evaluation: 'exact' };
    case 'constructed': return { ...common, family, rubric: 'Compare your reasoning with the criteria.', evaluation: 'self' };
    case 'procedure': return { ...common, family, steps: [{ id: crypto.randomUUID(), text: 'First step' }], reflection: 'Reflect on the process.', evaluation: 'self' };
  }
}
export function ActivityFields({ definition: d, set, composition }: { definition: ActivityDefinition; set: (value: ActivityDefinition) => void; composition?: Composition }) {
  return <fieldset><legend>Exercise definition</legend><Select label="Exercise family" value={d.family} values={['recall', 'singleChoice', 'constructed', 'procedure']} onChange={value => { const next = defaultActivity(value as ActivityDefinition['family']); if (next.family === 'singleChoice') next.correctOptionId = next.options[0].id; set({ ...next, prompt: d.prompt, hints: d.hints }); }} /><Text label="Exercise prompt" value={d.prompt} onChange={prompt => set({ ...d, prompt })} multiline />
    {d.family === 'recall' && <><Text label="Expected answer" value={d.answer} onChange={answer => set({ ...d, answer })} multiline /><Select label="Assessment" value={d.evaluation} values={['self', 'exact']} onChange={evaluation => set({ ...d, evaluation: evaluation as 'self' | 'exact' })} /></>}
    {d.family === 'singleChoice' && <><ol>{d.options.map((option, i) => <li key={option.id}><Text label={`Option ${i + 1}`} value={option.text} onChange={text => set({ ...d, options: d.options.map(o => o.id === option.id ? { ...o, text } : o) })} /><label><input type="radio" name={`correct-${d.options[0].id}`} checked={d.correctOptionId === option.id} onChange={() => set({ ...d, correctOptionId: option.id })} />Correct option {i + 1}</label><button type="button" disabled={d.options.length <= 2} onClick={() => set({ ...d, options: d.options.filter(o => o.id !== option.id), correctOptionId: d.correctOptionId === option.id ? d.options.find(o => o.id !== option.id)!.id : d.correctOptionId })}>Remove option {i + 1}</button></li>)}</ol><button type="button" disabled={d.options.length >= 16} onClick={() => set({ ...d, options: [...d.options, { id: crypto.randomUUID(), text: 'New option' }] })}>Add option</button></>}
    {d.family === 'constructed' && <><Text label="Assessment rubric" value={d.rubric} onChange={rubric => set({ ...d, rubric })} multiline /><label>Worked example material<select value={d.workedExampleBindingId ?? ''} onChange={e => { const next = { ...d }; if (e.target.value) next.workedExampleBindingId = e.target.value; else delete next.workedExampleBindingId; set(next); }}><option value="">No worked example</option>{composition?.bindings.filter(b => b.targetKind === 'resource').map((b, i) => <option value={b.bindingId} key={b.bindingId}>Material {i + 1}</option>)}</select></label><p>Use self assessment. No automatic semantic or code grading.</p></>}
    {d.family === 'procedure' && <><ol>{d.steps.map((step, i) => <li key={step.id}><Text label={`Step ${i + 1}`} value={step.text} onChange={text => set({ ...d, steps: d.steps.map(s => s.id === step.id ? { ...s, text } : s) })} /><button type="button" disabled={d.steps.length === 1} onClick={() => set({ ...d, steps: d.steps.filter(s => s.id !== step.id) })}>Remove step {i + 1}</button></li>)}</ol><button type="button" disabled={d.steps.length >= 64} onClick={() => set({ ...d, steps: [...d.steps, { id: crypto.randomUUID(), text: 'New step' }] })}>Add step</button><Text label="Reflection guidance" value={d.reflection} multiline onChange={reflection => set({ ...d, reflection })} /><p>Checklist selections do not verify physical performance.</p></>}
    <Text label="Hints (one per line)" multiline value={d.hints.join('\n')} onChange={text => set({ ...d, hints: text.split('\n').filter(Boolean) })} />
  </fieldset>;
}
const actionNames: ActionId[] = ['view.toggleSection', 'view.focusNode', 'source.open', 'response.update', 'study.hint', 'study.reveal', 'study.submit', 'study.next'];
export const nodeLabel = (node: LessonNode, index: number) => `${index + 1}. ${node.typeId}${'title' in node.props ? ` — ${String(node.props.title)}` : ''}`;
export function ActionFields({ value, onChange, composition, allowed = actionNames }: { value: ActionBinding; onChange: (action: ActionBinding) => void; composition: Composition; allowed?: ActionId[] }) {
  const targets = composition.nodes.filter(n => value.actionId === 'view.toggleSection' ? n.typeId === 'section' : value.actionId === 'view.focusNode' ? true : n.typeId === 'activity');
  return <fieldset><legend>Named action</legend><Select label="Action" value={value.actionId} values={allowed} onChange={id => onChange({ actionId: id as ActionId, version: 1, inputs: id === 'view.toggleSection' ? { expanded: true } : id === 'response.update' ? { value: { eventField: 'value' } } : {}, ...(id === 'source.open' ? { bindingId: composition.bindings.find(b => b.targetKind === 'source')?.bindingId ?? '' } : { targetNodeId: composition.nodes.find(n => id === 'view.toggleSection' ? n.typeId === 'section' : id === 'view.focusNode' ? true : n.typeId === 'activity')?.nodeId ?? '' }) })} />
    {value.actionId === 'source.open' ? <label>Original binding<select value={value.bindingId ?? ''} onChange={e => onChange({ ...value, bindingId: e.target.value })}><option value="">Choose original</option>{composition.bindings.filter(b => b.targetKind === 'source').map((b, i) => <option key={b.bindingId} value={b.bindingId}>Original {i + 1}</option>)}</select></label> : <label>Action target<select value={value.targetNodeId ?? ''} onChange={e => onChange({ ...value, targetNodeId: e.target.value })}><option value="">Choose component</option>{targets.map(n => <option key={n.nodeId} value={n.nodeId}>{nodeLabel(n, composition.nodes.indexOf(n))}</option>)}</select></label>}
    {value.actionId === 'view.toggleSection' && <Check label="Expanded after action" value={value.inputs.expanded === true} onChange={expanded => onChange({ ...value, inputs: { expanded } })} />}
    {value.actionId === 'response.update' && <><Select label="Response input" value={typeof value.inputs.value === 'object' ? 'event value' : 'constant'} values={['event value', 'constant']} onChange={kind => onChange({ ...value, inputs: { value: kind === 'event value' ? { eventField: 'value' } : '' } })} />{typeof value.inputs.value !== 'object' && <Text label="Constant response" value={String(value.inputs.value ?? '')} onChange={text => onChange({ ...value, inputs: { value: text } })} />}</>}
  </fieldset>;
}
export function ConditionFields({ value, set, depth = 0, label = 'Visibility' }: { value?: Condition; set: (condition?: Condition) => void; depth?: number; label?: string }) {
  const kinds = depth < 3 ? ['', 'literal', 'eq', 'compare', 'not', 'all', 'any'] : ['', 'literal', 'eq', 'compare'];
  const choose = (kind: string) => set(kind === '' ? undefined : kind === 'literal' ? { kind, value: true } : kind === 'eq' ? { kind, field: 'response.present', value: true } : kind === 'compare' ? { kind, field: 'assistance.hint', op: 'gte', value: 1 } : kind === 'not' ? { kind, expr: { kind: 'literal', value: true } } : { kind: kind as 'all' | 'any', exprs: [{ kind: 'literal', value: true }] });
  return <fieldset><legend>{label} condition</legend><Select label={`${label} condition type`} value={value?.kind ?? ''} values={kinds} onChange={choose} />
    {value?.kind === 'literal' && <Check label={`${label} visible`} value={value.value} onChange={checked => set({ ...value, value: checked })} />}
    {(value?.kind === 'eq' || value?.kind === 'compare') && <Select label={`${label} local field`} value={value.field} values={viewFields} onChange={field => set({ ...value, field: field as ViewField })} />}
    {value?.kind === 'eq' && <ScalarFields value={value.value} label={`${label} value`} set={scalar => set({ ...value, value: scalar })} />}
    {value?.kind === 'compare' && <><Select label={`${label} comparison`} value={value.op} values={['lt', 'lte', 'gt', 'gte']} onChange={op => set({ ...value, op: op as 'lt' })} /><label>{label} number<input type="number" value={value.value} onChange={e => set({ ...value, value: Number(e.target.value) })} /></label></>}
    {value?.kind === 'not' && <ConditionFields value={value.expr} depth={depth + 1} label={`${label} inner`} set={expr => set({ ...value, expr: expr ?? { kind: 'literal', value: true } })} />}
    {(value?.kind === 'all' || value?.kind === 'any') && <>{value.exprs.map((expr, i) => <div key={i}><ConditionFields value={expr} depth={depth + 1} label={`${label} ${i + 1}`} set={next => set({ ...value, exprs: value.exprs.map((old, index) => index === i ? next ?? { kind: 'literal', value: true } : old) })} /><button type="button" disabled={value.exprs.length <= 1} onClick={() => set({ ...value, exprs: value.exprs.filter((_, index) => index !== i) })}>Remove condition {i + 1}</button></div>)}<button type="button" disabled={value.exprs.length >= 4} onClick={() => set({ ...value, exprs: [...value.exprs, { kind: 'literal', value: true }] })}>Add condition</button></>}
    {depth === 0 && <p>Conditions read this component's local preview state. They do not grant access to answers or affect study policy.</p>}
  </fieldset>;
}
function ScalarFields({ value, set, label }: { value: Scalar; set: (value: Scalar) => void; label: string }) {
  const type = value === null ? 'null' : typeof value;
  return <><Select label={`${label} type`} value={type} values={['boolean', 'number', 'string', 'null']} onChange={type => set(type === 'boolean' ? true : type === 'number' ? 0 : type === 'null' ? null : '')} />{type === 'boolean' ? <Check label={label} value={value === true} onChange={set} /> : type === 'null' ? null : <label>{label}<input type={type === 'number' ? 'number' : 'text'} value={String(value)} onChange={e => set(type === 'number' ? Number(e.target.value) : e.target.value)} /></label>}</>;
}
export function InteractionForm({ node, registry, composition, disabled, save }: { node: LessonNode; registry?: RegistryDescriptor; composition: Composition; disabled: boolean; save: (on: LessonNode['on'], visible?: Condition) => void }) {
  const [on, setOn] = useState(node.on); const [visible, setVisible] = useState(node.visible);
  return <form onSubmit={(e: FormEvent) => { e.preventDefault(); save(on, visible); }}><fieldset disabled={disabled}><legend>Visibility and events</legend><ConditionFields value={visible} set={setVisible} />{registry?.events.map(event => { const binding = on[event]; const allowed = registry.eventSchemas[event].allowedActions; return <fieldset key={event}><legend>{event} event</legend><Check label={`Enable ${event} event`} value={!!binding} onChange={enabled => { const next = { ...on }; if (!enabled) delete next[event]; else next[event] = { actionId: allowed[0], version: 1, targetNodeId: node.nodeId, inputs: allowed[0] === 'response.update' ? { value: { eventField: 'value' } } : allowed[0] === 'view.toggleSection' ? { expanded: true } : {} }; setOn(next); }} />{binding && <ActionFields value={binding} allowed={allowed} composition={composition} onChange={action => setOn({ ...on, [event]: action })} />}</fieldset>; })}<button>Save visibility and events</button></fieldset></form>;
}
