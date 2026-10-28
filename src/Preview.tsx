import type { QualificationDraft } from './bridge';
import { Dialog } from './Dialog';
import { MathPreview } from './MathPreview';

export function Preview({ draft, onClose }: { draft: QualificationDraft; onClose: () => void }) {
  return <Dialog title="Reading preview" closeLabel="Close preview" onClose={onClose}>
    <article aria-label="Learning material">
      <h3>{draft.title}</h3>
      {draft.prose.split('\n').map((text, index) => <p key={index}>{text}</p>)}
      <h4>Mathematics</h4><MathPreview source={draft.formula} />
      <h4>Code example (read only; never executed)</h4><pre><code>{draft.code}</code></pre>
    </article>
  </Dialog>;
}
