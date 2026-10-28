import { useEffect, useRef } from 'react';
import katex from 'katex';

export function MathPreview({ source }: { source: string }) {
  const container = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!container.current) return;
    // This separate rendering budget also applies before Rust draft validation.
    if (source.length > 2048) {
      container.current.textContent = 'Formula too long to preview. Read or edit its source below.';
      return;
    }
    katex.render(source, container.current, { output: 'mathml', displayMode: true, trust: false, strict: 'error', throwOnError: false, maxExpand: 100, maxSize: 10 });
  }, [source]);
  return <><div ref={container} /><p>Formula source: <code>{source}</code></p></>;
}
