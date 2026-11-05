import { createElement, useId, useRef, useState, type ReactNode } from 'react';
import { DocumentEditor } from '../DocumentEditor';
import type { SavedDocument } from '../application/contentTypes';
import type { ActionBinding, ActivityRevision, Composition, Condition, LessonNode, SourceBindingRevision, ViewField } from './types';

export interface ResolvedLesson {
  resources: Record<string, SavedDocument | null>; activities: Record<string, ActivityRevision | null>; sources: Record<string, SourceBindingRevision | null>;
}
interface LocalState { response: string; checked: string[]; hint: number; revealed: boolean; submitted: boolean; expanded: boolean }
const initial = (): LocalState => ({ response: '', checked: [], hint: 0, revealed: false, submitted: false, expanded: true });
export const viewFields: ViewField[] = ['section.expanded', 'response.present', 'assistance.hint', 'assistance.revealed', 'preview.submitted'];
function field(state: LocalState, key: ViewField): string | number | boolean | undefined {
  switch (key) { case 'section.expanded': return state.expanded; case 'response.present': return !!state.response || state.checked.length > 0; case 'assistance.hint': return state.hint; case 'assistance.revealed': return state.revealed; case 'preview.submitted': return state.submitted; }
}
export function evaluateCondition(condition: Condition, state: LocalState): boolean | undefined {
  switch (condition.kind) {
    case 'literal': return typeof condition.value === 'boolean' ? condition.value : undefined;
    case 'eq': { const value = field(state, condition.field); return value === undefined || typeof value !== typeof condition.value ? undefined : value === condition.value; }
    case 'compare': { const value = field(state, condition.field); if (typeof value !== 'number' || !Number.isFinite(condition.value)) return undefined; return ({ lt: value < condition.value, lte: value <= condition.value, gt: value > condition.value, gte: value >= condition.value })[condition.op]; }
    case 'not': { const value = evaluateCondition(condition.expr, state); return value === undefined ? undefined : !value; }
    case 'all': case 'any': { const values = condition.exprs.map(c => evaluateCondition(c, state)); if (values.includes(undefined)) return undefined; return condition.kind === 'all' ? values.every(Boolean) : values.some(Boolean); }
  }
}
/** Trusted data renderer. Its disposable state is keyed by node occurrence, never activity ID. */
export function LessonRenderer({ composition, resources, activities, sources, onOpenSource }: ResolvedLesson & { composition: Composition; onOpenSource?: (source: SourceBindingRevision) => void }) {
  const instance = useId(); const [states, setStates] = useState<Record<string, LocalState>>({}); const [notice, setNotice] = useState(''); const refs = useRef(new Map<string, HTMLDivElement>());
  const get = (id: string) => states[id] ?? initial();
  const update = (id: string, patch: Partial<LocalState>) => setStates(old => ({ ...old, [id]: { ...(old[id] ?? initial()), ...patch } }));
  const binding = (id: unknown) => composition.bindings.find(b => b.bindingId === id);
  function action(action: ActionBinding, origin: string, value?: string | boolean) {
    const target = action.targetNodeId ?? origin; if (!composition.nodes.some(n => n.nodeId === target)) { setNotice('The action target is unavailable.'); return; }
    const state = get(target);
    switch (action.actionId) {
      case 'view.toggleSection': { const input = action.inputs.expanded; update(target, { expanded: typeof input === 'boolean' ? input : typeof input === 'object' && input?.eventField === 'value' && typeof value === 'boolean' ? value : state.expanded }); break; }
      case 'view.focusNode': refs.current.get(target)?.focus(); break;
      case 'response.update': { const input = action.inputs.value; update(target, { response: typeof input === 'object' && input?.eventField === 'value' ? String(value ?? '') : typeof input === 'string' ? input : '' }); break; }
      case 'study.hint': update(target, { hint: state.hint + 1 }); break;
      case 'study.reveal': update(target, { revealed: true }); break;
      case 'study.submit': update(target, { submitted: true }); break;
      case 'study.next': setNotice('Real study sessions are not available here. This is a preview only.'); break;
      case 'source.open': { const b = binding(action.bindingId); const source = b && sources[b.revisionId]; if (source && onOpenSource) onOpenSource(source); else setNotice('The pinned original is unavailable.'); break; }
    }
  }
  function render(id: string, ancestors: string[] = [], level = 2): ReactNode {
    const node = composition.nodes.find(n => n.nodeId === id); if (!node || ancestors.includes(id) || ancestors.length > 12) return <p role="note" key={id}>Unavailable component.</p>;
    const state = get(id); const props = node.props as Record<string, unknown>; const known = ['stack', 'columns', 'section', 'content', 'source', 'activity', 'button', 'status'].includes(node.typeId) && node.typeVersion === 1;
    if (!known) return <div key={id} role="note">Unsupported component preserved. Replace or remove it in the editor to repair the lesson.</div>;
    if (node.visible) { const visible = evaluateCondition(node.visible, state); if (visible === undefined) return <p key={id} role="note">Visibility condition unavailable. Review this component's local field and value.</p>; if (!visible) return null; }
    const children = (slot: string) => (node.slots[slot] ?? []).map(child => render(child, [...ancestors, id], node.typeId === 'stack' ? Number(props.headingLevel) : node.typeId === 'section' ? Math.min(6, level + 1) : level));
    const b = binding(props.bindingId); let body: ReactNode;
    switch (node.typeId) {
      case 'stack': body = <div className={`lesson-stack gap-${String(props.gap)}`}>{children('body')}</div>; break;
      case 'columns': body = <div className={`lesson-columns ratio-${String(props.ratio)}`}><div>{children('primary')}</div><div>{children('secondary')}</div></div>; break;
      case 'section': body = <section>{createElement(`h${Math.min(6, Math.max(1, level))}`, null, String(props.title))}{props.collapsible === true && <button aria-expanded={state.expanded} onClick={() => { const expanded = !state.expanded; const on = node.on.toggle; if (on) action(on, id, expanded); else update(id, { expanded }); }}> {state.expanded ? 'Collapse' : 'Expand'} {String(props.title)}</button>}{state.expanded && children('body')}</section>; break;
      case 'content': { const resource = b && resources[b.revisionId]; let document = resource?.document; const range = props.blockRange as { startBlockId: string; endBlockId: string } | undefined; if (document && range) { const start = document.blocks.findIndex(block => block.id === range.startBlockId); const end = document.blocks.findIndex(block => block.id === range.endBlockId); document = start >= 0 && end >= start ? { ...document, blocks: document.blocks.slice(start, end + 1) } : undefined; } body = document ? <DocumentEditor document={document} readOnly passive label="Lesson reading" onChange={() => {}} /> : <p>Pinned material unavailable.</p>; break; }
      case 'source': { const source = b && sources[b.revisionId]; body = source ? <aside><h3>{source.filename}</h3><p>{source.attribution ?? 'Attribution unavailable'} · {source.license ?? 'License unknown'}</p><p>Extraction: not extracted. Read coverage: unknown.</p>{source.locator && <p>Location: {source.locator}</p>}{source.declaredUsedRanges.length > 0 && <p>Declared ranges: {source.declaredUsedRanges.join(', ')}. These are selections, not observed coverage.</p>}{source.warnings.map((warning, i) => <p key={i}>{warning}</p>)}<button onClick={() => { if (node.on.openSource) action(node.on.openSource, id); else if (onOpenSource) onOpenSource(source); else setNotice('The original viewer is unavailable.'); }}>Open original {source.filename}</button></aside> : <p>Pinned original unavailable.</p>; break; }
      case 'activity': { const activity = b && activities[b.revisionId]; const example = activity?.definition.family === 'constructed' && binding(activity.definition.workedExampleBindingId); const workedExample = example && resources[example.revisionId]?.document; body = activity ? <Activity node={node} activity={activity} state={state} update={patch => update(id, patch)} event={(name, value) => { const on = node.on[name]; if (on && (name !== 'response' || on.actionId !== 'study.submit')) action(on, id, value); }} name={`${instance}-${id}`} workedExample={workedExample || undefined} /> : <p>Pinned exercise unavailable.</p>; break; }
      case 'button': body = <button className={`lesson-action ${String(props.style)}`} onClick={() => action(node.on.activate ?? props.action as ActionBinding, id)}>{String(props.label)}</button>; break;
      case 'status': { const target = typeof props.targetNodeId === 'string' ? props.targetNodeId : id; const targetExists = composition.nodes.some(n => n.nodeId === target); body = <output>{String(props.label)}: {targetExists ? String(field(get(target), props.field as ViewField) ?? 'Unavailable') : 'Source unavailable'}{!props.targetNodeId && <span> — No source selected; showing this component's local view state.</span>}</output>; break; }
    }
    return <div key={id} tabIndex={-1} ref={element => { if (element) refs.current.set(id, element); else refs.current.delete(id); }} data-lesson-node={id}>{body}</div>;
  }
  return <div className="lesson-renderer"><p className="lesson-preview-note">Preview only · Simulated answers and assistance. No study history or progress is recorded.</p>{notice && <p role="status">{notice}</p>}{render(composition.rootNodeId)}</div>;
}
function Activity({ node, activity, state, update, event, name, workedExample }: { node: LessonNode; activity: ActivityRevision; state: LocalState; update: (patch: Partial<LocalState>) => void; event: (name: string, value?: string) => void; name: string; workedExample?: SavedDocument['document'] }) {
  const d = activity.definition;
  const change = (response: string) => { update({ response, submitted: false }); event('response', response); };
  const reveal = d.family === 'recall' ? d.answer : d.family === 'singleChoice' ? d.options.find(o => o.id === d.correctOptionId)?.text : d.family === 'constructed' ? d.rubric : d.reflection;
  return <fieldset className={`lesson-exercise ${String((node.props as Record<string, unknown>).presentation ?? 'default')}`}><legend>{d.prompt}</legend>
    {d.family === 'singleChoice' ? d.options.map(option => <label key={option.id}><input type="radio" name={name} checked={state.response === option.id} onChange={() => change(option.id)} />{option.text}</label>) : d.family === 'procedure' ? <ol>{d.steps.map(step => <li key={step.id}><label><input type="checkbox" checked={state.checked.includes(step.id)} onChange={e => { update({ checked: e.target.checked ? [...state.checked, step.id] : state.checked.filter(id => id !== step.id), submitted: false }); }} />{step.text}</label></li>)}</ol> : <label>Your response<textarea value={state.response} onChange={e => change(e.target.value)} /></label>}
    {d.family === 'procedure' && <label>Your reflection<textarea value={state.response} onChange={e => change(e.target.value)} /></label>}
    <button disabled={state.hint >= d.hints.length} onClick={() => { update({ hint: state.hint + 1 }); event('hint'); }}>Show preview hint</button><button onClick={() => { update({ revealed: true }); event('reveal'); }}>Reveal preview guidance</button><button onClick={() => { update({ submitted: true }); event('submit'); }}>Check preview response</button>
    {d.hints.slice(0, state.hint).map((hint, i) => <p key={i}>Hint: {hint}</p>)}{state.revealed && <p>Preview guidance: {reveal}</p>}
    {state.revealed && d.family === 'constructed' && d.workedExampleBindingId && (workedExample ? <aside><h3>Worked example</h3><DocumentEditor document={workedExample} readOnly passive label="Worked example reading" onChange={() => {}} /></aside> : <p>Pinned worked example unavailable.</p>)}
    {state.submitted && <p role="status">{d.evaluation === 'exact' ? state.response === (d.family === 'singleChoice' ? d.correctOptionId : d.family === 'recall' ? d.answer : '') ? 'Exact answer matched.' : 'Exact answer did not match.' : 'Compare your work with the guidance. Assessment remains your choice; no result is recorded.'}</p>}
  </fieldset>;
}
