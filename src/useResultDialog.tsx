import { useState } from 'react';
import { Dialog } from './Dialog';

/** Owns one explicit result and the exact control to return to on dismissal. */
export function useResultDialog() {
  const [result, setResult] = useState<{ message: string; origin: HTMLButtonElement } | null>(null);
  return {
    show: (message: string, origin: HTMLButtonElement) => setResult({ message, origin }),
    dialog: result && <Dialog title="Operation result" closeLabel="Close result" description={result.message}
      onClose={() => { setResult(null); result.origin.focus(); }} />,
  };
}
