import { useEffect, useRef, useState, type FormEvent } from 'react';
import type { Command, Entity, Library, RelationKind, Step, Unit } from './contentBridge';

type Perform = (command: Command) => Promise<Library | null>;
export function NameForm({ label, action, onSubmit }: { label: string; action: string; onSubmit: (value: string) => Promise<unknown> }) {
  const [value, setValue] = useState('');
  async function submit(event: FormEvent) { event.preventDefault(); if (value.trim() && await onSubmit(value)) setValue(''); }
  return <form onSubmit={event => void submit(event)}><label>{label}<input value={value} maxLength={256} required onChange={event => setValue(event.target.value)} /></label><button>{action}</button></form>;
}
export function Options({ items }: { items: Record<string, Entity> }) {
  return <>{Object.values(items).filter(x => !x.archived).map(x => <option key={x.id} value={x.id}>{x.title}</option>)}</>;
}
export function UnitForms({ data, perform }: { data: Library; perform: Perform }) {
  const [from, setFrom] = useState(''); const [to, setTo] = useState(''); const [kind, setKind] = useState<RelationKind>('prerequisite');
  const [editing, setEditing] = useState('');
  return <section><h3>Learning units and links</h3>
    <NameForm label="New learning unit" action="Create unit" onSubmit={title => perform({ type: 'createUnit', id: crypto.randomUUID(), title, objective: title, goalIds: [], resourceIds: [] })} />
    <label>Edit learning unit<select value={editing} onChange={e => setEditing(e.target.value)}><option value="">Choose unit</option><Options items={data.units} /></select></label>
    {data.units[editing] && <UnitEditor key={editing} unit={data.units[editing]} data={data} perform={perform} />}
    <form onSubmit={event => { event.preventDefault(); void perform({ type: 'relate', id: crypto.randomUUID(), kind, from, to }); }}>
      <label>From unit<select required value={from} onChange={e => setFrom(e.target.value)}><option value="">Choose unit</option><Options items={data.units} /></select></label>
      <label>Link type<select value={kind} onChange={e => setKind(e.target.value as RelationKind)}><option value="prerequisite">Prerequisite</option><option value="related">Related</option><option value="reference">Reference</option></select></label>
      <label>To unit<select required value={to} onChange={e => setTo(e.target.value)}><option value="">Choose unit</option><Options items={data.units} /></select></label><button>Add link</button>
    </form>
    <ul>{Object.values(data.relations).map(link => <li key={link.id}>{data.units[link.from]?.title} → {link.kind} → {data.units[link.to]?.title} <button type="button" onClick={() => void perform({ type: 'removeRelation', id: link.id })}>Remove link</button></li>)}</ul>
  </section>;
}
function UnitEditor({ unit, data, perform }: { unit: Unit; data: Library; perform: Perform }) {
  const [title, setTitle] = useState(unit.title); const [objective, setObjective] = useState(unit.objective);
  const [goalIds, setGoals] = useState(unit.goalIds); const [resourceIds, setResources] = useState(unit.resourceIds);
  return <form onSubmit={event => { event.preventDefault(); void perform({ type: 'updateUnit', id: unit.id, title, objective, goalIds, resourceIds }); }}>
    <label>Unit title<input required maxLength={256} value={title} onChange={e => setTitle(e.target.value)} /></label>
    <label>Unit objective<textarea required maxLength={4096} value={objective} onChange={e => setObjective(e.target.value)} /></label>
    <LinkChoices label="Goal" items={data.goals} selected={goalIds} change={setGoals} />
    <LinkChoices label="Material" items={data.resources} selected={resourceIds} change={setResources} />
    <button>Save unit</button>
  </form>;
}
function LinkChoices({ label, items, selected, change }: { label: string; items: Record<string, Entity>; selected: string[]; change: (ids: string[]) => void }) {
  return <fieldset><legend>{label} links</legend>{Object.values(items).filter(item => !item.archived).map(item => <label key={item.id}><input type="checkbox" checked={selected.includes(item.id)} onChange={event => change(event.target.checked ? [...selected, item.id] : selected.filter(id => id !== item.id))} />{label}: {item.title}</label>)}</fieldset>;
}
export function GoalForm({ subjectId, perform }: { subjectId: string; perform: Perform }) {
  const [title, setTitle] = useState(''); const [criteria, setCriteria] = useState('');
  async function submit(event: FormEvent) { event.preventDefault(); if (await perform({ type: 'createGoal', id: crypto.randomUUID(), subjectId, title, criteria })) { setTitle(''); setCriteria(''); } }
  return <form onSubmit={event => void submit(event)}><label>New goal<input required maxLength={256} value={title} onChange={e => setTitle(e.target.value)} /></label><label>Goal criteria<textarea maxLength={4096} value={criteria} onChange={e => setCriteria(e.target.value)} /></label><button>Add goal</button></form>;
}
export function PathForms({ data, perform }: { data: Library; perform: Perform }) {
  const [path, setPath] = useState(''); const [unit, setUnit] = useState('');
  function append() { const current = data.paths[path]; if (current && unit) void perform({ type: 'setPathSteps', id: path, steps: [...current.steps, { id: crypto.randomUUID(), unitId: unit }] }); }
  return <section><h3>Learning paths</h3>
    <NameForm label="New path" action="Create path" onSubmit={title => perform({ type: 'createPath', id: crypto.randomUUID(), title, steps: [] })} />
    <label>Path<select value={path} onChange={e => setPath(e.target.value)}><option value="">Choose path</option><Options items={data.paths} /></select></label>
    <label>Next unit<select value={unit} onChange={e => setUnit(e.target.value)}><option value="">Choose unit</option><Options items={data.units} /></select></label>
    <button type="button" disabled={!path || !unit} onClick={append}>Append unit</button>
    <ol>{data.paths[path]?.steps.map((step: Step, index) => <li key={step.id}>{data.units[step.unitId]?.title}<ReorderControls title={`step ${index + 1}: ${data.units[step.unitId]?.title}`} id={step.id} ids={data.paths[path].steps.map(item => item.id)} reorder={ids => void perform({ type: 'setPathSteps', id: path, steps: ids.flatMap(id => data.paths[path].steps.filter(item => item.id === id)) })} /><button type="button" onClick={() => void perform({ type: 'setPathSteps', id: path, steps: data.paths[path].steps.filter(x => x.id !== step.id) })}>Remove step</button></li>)}</ol>
  </section>;
}
export function ReorderControls({ title, id, ids, reorder }: { title: string; id: string; ids: string[]; reorder: (ids: string[]) => void }) {
  const index = ids.indexOf(id);
  const container = useRef<HTMLSpanElement>(null); const returnFocus = useRef(false); const order = JSON.stringify(ids);
  useEffect(() => { if (returnFocus.current) { container.current?.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus(); returnFocus.current = false; } }, [order]);
  function move(delta: number) { const ordered = [...ids]; [ordered[index], ordered[index + delta]] = [ordered[index + delta], ordered[index]]; returnFocus.current = true; reorder(ordered); }
  return <span ref={container}><button type="button" aria-label={`Move ${title} earlier`} disabled={index <= 0} onClick={() => move(-1)}>Move earlier</button><button type="button" aria-label={`Move ${title} later`} disabled={index < 0 || index === ids.length - 1} onClick={() => move(1)}>Move later</button></span>;
}
