import type { ExtensionAdmission } from './contracts';

/** Deliberately no action bridge, generated markup or executable content. */
export function ExtensionPlaceholder({ admission }: { admission: ExtensionAdmission }) {
  return <aside role="note" aria-label="Interactive extension unavailable">
    <p>{admission.status === 'inert' ? 'Interactive extension preserved. Execution is unavailable.' : 'Interactive extension rejected for this course.'}</p>
    <p>Existing lesson components remain available. Original extension data can be retained for recovery; it does not record learning results.</p>
  </aside>;
}
