import { useState } from 'react';
import { failureMessage, isConfirmedRejection } from '../application/errors';
import type { ConfirmPackage, ImportResult, PackageKind, PackagePort, PackageReview, PackageSummary } from '../application/packageTypes';
function Summary({ value }: { value: PackageSummary }) {
 const history = typeof value.manifest.sections === 'object' && value.manifest.sections !== null && 'studyHistory' in value.manifest.sections;
 return <div>
  <p>{value.kind === 'nativeFull' ? 'Full native recovery for the selected course, including private history and content versions.' : history ? 'Portable content includes opted-in study responses, judgments and study receipts. Saved study drafts, earlier draft snapshots in study receipts, review schedules, policy, practical reminders and frozen assessments are included. Notes, authoring lesson drafts, jobs and preferences are omitted; this is not full recovery.' : 'Content only: does not provide full recovery. Authored content-version history is included; private learning history, notes, drafts, jobs and preferences are omitted.'}</p>
  <p>Scope: {value.manifest.scope}. Source course: {value.manifest.sourceCourseId}.</p>
  <p>{value.manifest.complete === false ? 'Incomplete: some originals are unavailable.' : 'Originals and completeness are declared below. Omitted originals cannot be recovered from this package.'}</p>
  {value.kind === 'portableContent' && <p>Originals requested: {value.manifest.originalsRequested === true ? 'yes' : 'no'}. Intentionally omitted assets: {Array.isArray(value.manifest.intentionallyOmittedAssets) ? value.manifest.intentionallyOmittedAssets.length : 0}. Unavailable assets: {Array.isArray(value.manifest.unavailableAssets) ? value.manifest.unavailableAssets.length : 0}.</p>}
  {value.kind === 'nativeFull' && <p>Unavailable originals: {value.manifest.missingAssets?.length ?? 0}. All persisted content-version history and private recovery state are included.</p>}
  <details><summary>Manifest details, versions, originals and omissions</summary><pre>{JSON.stringify(value.manifest, null, 2)}</pre></details>
  <p>Omissions: {value.manifest.omissions.join('; ') || 'None declared'}.</p>
 </div>;
}
export function PackagePanel({ port, onImported }: { port: PackagePort; onImported?: (result: ImportResult) => void | Promise<void> }) {
 const [history,setHistory]=useState(false);const [originals,setOriginals]=useState(false);const [busy,setBusy]=useState(false);
 const [review,setReview]=useState<PackageReview|null>(null);const [exported,setExported]=useState<PackageSummary|null>(null);
 const [title,setTitle]=useState('');const [allow,setAllow]=useState(false);
 const [pending,setPending]=useState<ConfirmPackage|null>(null);const [result,setResult]=useState<ImportResult|null>(null);
 const [error,setError]=useState('');const [status,setStatus]=useState('');
 async function run(action:()=>Promise<void>) {setBusy(true);setError('');setStatus('');try{await action();}catch(e){setError(failureMessage(e));}finally{setBusy(false);}}
 async function exportPackage(kind:PackageKind) {await run(async()=>{const value=await port.exportPackage(kind,originals,kind==='portableContent'&&history);setExported(value);setStatus(value?'Exported selected-course package.':'Package export cancelled.');});}
 async function inspect() {await run(async()=>{const value=await port.inspect();setReview(value);setAllow(false);setResult(null);if(!value)setStatus('Package inspection cancelled.');});}
 async function confirm() {
  if(!review)return;
  const request=pending??{reviewId:review.reviewId,operationId:crypto.randomUUID(),title,allowIncomplete:allow};
  setPending(request);
  await run(async()=>{try{const value=await port.confirm(request);setResult(value);setPending(null);setStatus(`Created new course ${value.report.courseId}. Existing courses were preserved.`);await onImported?.(value);}catch(e){if(isConfirmedRejection(e))setPending(null);throw e;}});
 }
 async function discard() {if(!review)return;await run(async()=>{await port.discard(review.reviewId);setReview(null);setPending(null);setResult(null);});}
 const locked=busy||pending!==null;
 return <section aria-label="Course packages"><h2>Course packages</h2>
  <p>Exports capture this course before opening the native file chooser. Imports always create a new course. Files remain passive; Memoquill never opens or executes their contents.</p>
  <button disabled={locked||review!==null} onClick={()=>void exportPackage('nativeFull')}>Export full recovery package</button>
  <label><input type="checkbox" checked={originals} disabled={locked} onChange={e=>setOriginals(e.target.checked)}/>Include originals in shareable package</label>
  <label><input type="checkbox" checked={history} disabled={locked} onChange={e=>setHistory(e.target.checked)}/>Include my study responses and assessment history in shareable package</label>
  {history&&<p>This includes private saved responses, saved study drafts, earlier draft snapshots in study receipts, help declarations, timestamps, judgments, recall schedules, practical reminders and frozen assessments from this course.</p>}
  <button disabled={locked||review!==null} onClick={()=>void exportPackage('portableContent')}>Export shareable content package</button>
  <button disabled={locked||review!==null} onClick={()=>void inspect()}>Choose package to inspect</button>
  {exported&&<section aria-label="Exported package"><Summary value={exported}/></section>}
  {review&&<section aria-label="Package review"><h3>Package review</h3><Summary value={review}/>
   <label>New course title<input maxLength={256} value={title} disabled={locked||result!==null} onChange={e=>setTitle(e.target.value)}/></label>
   {review.kind==='nativeFull'&&review.manifest.complete===false&&<label><input type="checkbox" checked={allow} disabled={locked} onChange={e=>setAllow(e.target.checked)}/>Allow incomplete recovery</label>}
   {!result&&<button disabled={busy||!title.trim()||(review.kind==='nativeFull'&&review.manifest.complete===false&&!allow)} onClick={()=>void confirm()}>{pending?'Retry same package import':'Confirm import into new course'}</button>}
   <button disabled={busy||pending!==null} onClick={()=>void discard()}>Discard package review</button>
  </section>}
  {result&&<section aria-label="Package import result"><h3>New course and identity remaps</h3><p>Source course {result.report.sourceCourseId} → new course {result.report.courseId}. Internal authored identifiers are preserved inside the isolated new course.</p><pre>{JSON.stringify(result.report,null,2)}</pre></section>}
  {busy&&<p role="status">Working with selected-course package…</p>}{status&&<p role="status">{status}</p>}{error&&<p role="alert">{error}</p>}
 </section>;
}
