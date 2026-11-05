import type { ButtonHTMLAttributes, ReactNode } from 'react';

/** Shared native controls retain browser keyboard and disabled semantics. */
export function ActionButton(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button type="button" {...props} />;
}
export function StatusMessage({ children }: { children: ReactNode }) {
  return <p role="status" aria-live="polite" aria-atomic="true">{children}</p>;
}
