import type { ActivityDefinition } from './types';

export function InstructionalSupport({ definition, preview = false }: { definition: ActivityDefinition; preview?: boolean }) {
  if (definition.family !== 'constructed' || definition.version === 1) return null;
  const instruction = definition.instruction;
  const Heading = preview ? 'h3' : 'h4';
  return <aside aria-label={preview ? 'Instructional support preview' : 'Instructional support'}>
    <Heading>{instruction.phase === 'errorRepair' ? 'Deliberately flawed example — find and repair the error' : instruction.phase === 'workedExample' ? 'Worked example — explain the reasoning' : 'Completion — supply the missing steps'}</Heading>
    {instruction.phase === 'errorRepair' && <p>{instruction.label}</p>}
    <pre>{instruction.material}</pre>
    {instruction.phase === 'errorRepair' ? <>
      <p>Review recorded by {instruction.review.reviewer}. Reference: {instruction.review.reference}.</p>
      <p>Review details are author-supplied; they do not establish independent factual verification. The expected correction stays hidden until you reveal the reference.</p>
    </> : <p>{preview ? 'Assisted practice preview; no study evidence is recorded.' : 'This response uses a shown solution or scaffold. It is recorded as assisted practice.'}</p>}
    {preview && instruction.phase === 'errorRepair' && <p>Preview does not record study evidence.</p>}
  </aside>;
}
