import { useEffect, useId, useRef, type ReactNode } from 'react';

/** Read-only fixture dialog: exactly one interactive control. */
export function Dialog({ title, closeLabel, description, children, onClose }: {
  title: string; closeLabel: string; description?: string;
  children?: ReactNode; onClose: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const close = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const descriptionId = useId();
  useEffect(() => {
    dialog.current?.showModal();
    close.current?.focus();
  }, []);
  return <dialog ref={dialog} aria-labelledby={titleId}
    aria-describedby={description ? descriptionId : undefined} onClose={onClose}
    onKeyDown={event => {
      // Native GTK/WebKit traversal needs an explicit loop in this bounded
      // single-control fixture. Reassess before adding any other controls.
      if (event.key === 'Tab') { event.preventDefault(); close.current?.focus(); }
    }}>
    <button ref={close} onClick={() => dialog.current?.close()}>{closeLabel}</button>
    <h2 id={titleId}>{title}</h2>
    {description && <p id={descriptionId}>{description}</p>}
    {children}
  </dialog>;
}
