import { useEffect, useState, type FormEvent } from 'react';
import { ApplicationProvider, type ApplicationPorts } from '../application/ApplicationProvider';
import type { ImportResult } from '../application/packageTypes';
import { PackagePanel } from './PackagePanel';
import { LibraryApp } from '../LibraryApp';
import { failureMessage, isConfirmedRejection } from '../application/errors';
import { createNativeCoursePorts, nativeCoursePort } from './nativeCoursePort';
import type { CourseCatalogue, CoursePort } from './types';

type Action = { type: 'list' } | { type: 'select'; id: string } | { type: 'create'; operationId: string; title: string };
export function CourseApp({ coursePort = nativeCoursePort, makePorts = createNativeCoursePorts }: {
  coursePort?: CoursePort; makePorts?: (id: string) => ApplicationPorts;
}) {
  const [imported, setImported] = useState<ImportResult | null>(null);
  const [catalogue, setCatalogue] = useState<CourseCatalogue | null>(null);
  const [mounted, setMounted] = useState<{ id: string; ports: ApplicationPorts }[]>([]);
  const [title, setTitle] = useState(''); const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<Action | null>(null); const [error, setError] = useState('');
  function accept(value: CourseCatalogue) {
    setCatalogue(value);
    setMounted(previous => previous.some(item => item.id === value.selectedCourseId) ? previous : [...previous, { id: value.selectedCourseId, ports: makePorts(value.selectedCourseId) }]);
  }
  useEffect(() => {
    let active = true;
    coursePort.list().then(value => { if (active) accept(value); }).catch(reason => { if (active) { setError(failureMessage(reason)); setPending({ type: 'list' }); } });
    return () => { active = false; };
  }, [coursePort]);
  async function run(action: Action) {
    setBusy(true); setPending(action); setError('');
    try {
      const value = action.type === 'list' ? await coursePort.list() : action.type === 'select' ? await coursePort.select(action.id) : await coursePort.create(action.operationId, action.title);
      accept(value); setPending(null); if (action.type === 'create') setTitle('');
    } catch (reason) { setError(failureMessage(reason)); if (isConfirmedRejection(reason)) setPending(null); }
    finally { setBusy(false); }
  }
  function create(event: FormEvent) {
    event.preventDefault(); if (title.trim() && !busy && !pending) void run({ type: 'create', operationId: crypto.randomUUID(), title });
  }
  return <div className="course-shell">
    <aside className="course-sidebar"><h2>Courses</h2><nav aria-label="Courses"><ul>{catalogue?.courses.map(course => <li key={course.id}>
      <button disabled={busy || pending !== null} aria-current={catalogue.selectedCourseId === course.id ? 'page' : undefined} onClick={() => void run({ type: 'select', id: course.id })}>{course.title}</button>
    </li>)}</ul></nav>
    <form onSubmit={create}><label>New course<input value={title} maxLength={256} disabled={busy || pending !== null} onChange={event => setTitle(event.target.value)} /></label><button disabled={!title.trim() || busy || pending !== null}>Create course</button></form>
    {busy && <p role="status">Updating courses…</p>}{error && <p role="alert">{error}</p>}
    {pending && !busy && <button onClick={() => void run(pending)}>Retry course action</button>}
    {!catalogue && !error && <p role="status">Opening courses…</p>}
    </aside>
    <div className="course-workspace">{imported && <section aria-label="Latest package import result"><p role="status">Created new course {imported.report.courseId}. Existing courses were preserved.</p><p>Course remap: {imported.report.sourceCourseId} → {imported.report.courseId}.</p><pre>{JSON.stringify(imported.report, null, 2)}</pre></section>}{mounted.map(course => <div key={course.id} hidden={catalogue?.selectedCourseId !== course.id} inert={catalogue?.selectedCourseId !== course.id} data-course-id={course.id}>
      <ApplicationProvider ports={course.ports} active={catalogue?.selectedCourseId === course.id}>{course.ports.packages && <PackagePanel port={course.ports.packages} onImported={async result => { setImported(result); accept(await coursePort.list()); }} />}<LibraryApp /></ApplicationProvider>
    </div>)}</div>
  </div>;
}
