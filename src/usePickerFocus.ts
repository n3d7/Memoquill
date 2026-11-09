import { useCallback, useEffect, useRef, useState } from 'react';

/** Restores focus only after React commits the re-enabled picker trigger. */
export function usePickerFocus(pending: boolean) {
  const [element, setElement] = useState<HTMLButtonElement | null>(null);
  const trigger = useCallback((value: HTMLButtonElement | null) => { setElement(value); }, []);
  const requested = useRef(false);
  useEffect(() => {
    if (!pending && requested.current && element) {
      requested.current = false;
      if (!element.closest('[hidden], [inert]')) element.focus();
    }
  }, [pending, element]);
  return { trigger, requestFocus: () => { requested.current = true; } };
}
