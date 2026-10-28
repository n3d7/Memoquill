import { useEffect, useRef } from 'react';

/** Restores focus only after React commits the re-enabled picker trigger. */
export function usePickerFocus(pending: boolean) {
  const trigger = useRef<HTMLButtonElement>(null);
  const requested = useRef(false);
  useEffect(() => {
    if (!pending && requested.current) {
      requested.current = false;
      trigger.current?.focus();
    }
  }, [pending]);
  return { trigger, requestFocus: () => { requested.current = true; } };
}
