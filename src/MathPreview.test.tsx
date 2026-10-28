import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { MathPreview } from './MathPreview';

test('renders native MathML with its source and recovers after an oversized input', () => {
  const { container, rerender } = render(<MathPreview source={'x'.repeat(2049)} />);
  expect(screen.getByText(/Formula too long/)).toBeVisible();
  expect(container.querySelector('math')).toBeNull();
  rerender(<MathPreview source="x^2" />);
  expect(container.querySelector('math')).not.toBeNull();
  expect(screen.getByText(/Formula source:/)).toHaveTextContent('x^2');
});

test('renders malformed input as a passive error instead of throwing', () => {
  const { container } = render(<MathPreview source={String.raw`\invalidmacro{`} />);
  expect(container.querySelector('.katex-error')).not.toBeNull();
  expect(container.querySelector('script')).toBeNull();
});
