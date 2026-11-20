import { useId, useMemo, useState } from 'react';
import type { Entity } from './contentTypes';

interface Props {
  label: string;
  placeholder: string;
  items: Readonly<Record<string, Pick<Entity, 'title' | 'archived'>>>;
  value: string;
  onChange: (id: string) => void;
  required?: boolean;
}

const PAGE_SIZE = 40;

export function EntitySelect({ label, placeholder, items, value, onChange, required }: Props) {
  const statusId = useId();
  const [filter, setFilter] = useState('');
  const [page, setPage] = useState(0);
  const active = useMemo(() => Object.entries(items).flatMap(([id, item]) =>
    item.archived ? [] : [{ id, title: item.title }]), [items]);
  const paged = active.length > PAGE_SIZE;
  const matches = useMemo(() => {
    const query = filter.trim().toLowerCase();
    return paged && query ? active.filter(item => item.title.toLowerCase().includes(query)) : active;
  }, [active, filter, paged]);
  const last = Math.max(0, Math.ceil(matches.length / PAGE_SIZE) - 1);
  const current = Math.min(page, last);
  const start = current * PAGE_SIZE;
  const visible = paged ? matches.slice(start, start + PAGE_SIZE) : matches;
  const selected = items[value];
  const pinned = selected && !selected.archived && !visible.some(item => item.id === value);

  return <div>
    {paged && <>
      <label>Filter {label.toLowerCase()}<input type="search" value={filter} onChange={event => {
        setFilter(event.target.value);
        setPage(0);
      }} /></label>
      <p role="status" id={statusId}>
        {matches.length ? `Showing ${start + 1}–${start + visible.length} of ${matches.length} choices.` : 'No matching choices.'}
      </p>
      {last > 0 && <div className="actions" role="group" aria-label={`${label} pages`}>
        <button type="button" disabled={current === 0} onClick={() => setPage(0)}>First choices</button>
        <button type="button" disabled={current === 0} onClick={() => setPage(current - 1)}>Previous choices</button>
        <button type="button" disabled={current === last} onClick={() => setPage(current + 1)}>Next choices</button>
        <button type="button" disabled={current === last} onClick={() => setPage(last)}>Last choices</button>
      </div>}
    </>}
    <label>{label}<select required={required} aria-describedby={paged ? statusId : undefined}
      value={selected && !selected.archived ? value : ''} onChange={event => onChange(event.target.value)}>
      <option value="">{placeholder}</option>
      {pinned && <option value={value}>{selected.title}</option>}
      {visible.map(item => <option key={item.id} value={item.id}>{item.title}</option>)}
    </select></label>
  </div>;
}
