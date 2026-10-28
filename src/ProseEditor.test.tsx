import { fireEvent, render, screen } from '@testing-library/react';
import { expect, test, vi } from 'vitest';
import { ProseEditor } from './ProseEditor';

test('plain text paste preserves Unicode, emits text, and never inserts active HTML', () => {
  const onChange = vi.fn();
  render(<ProseEditor initialText="" onChange={onChange} />);
  const editor = screen.getByRole('textbox', { name: 'Learning notes' });
  const getData = vi.fn((type: string) => type === 'text/plain' ? 'café 世界' : '<img src="https://example.com/private">');
  fireEvent.paste(editor, { clipboardData: { getData } });
  expect(editor).toHaveTextContent('café 世界');
  expect(onChange).toHaveBeenLastCalledWith('café 世界');
  expect(editor.querySelector('img')).toBeNull();
  expect(getData).toHaveBeenCalledTimes(1);
});

test('keyboard bold, undo and redo operate on the editor without a mouse', () => {
  render(<ProseEditor initialText="" onChange={vi.fn()} />);
  const editor = screen.getByRole('textbox', { name: 'Learning notes' });
  fireEvent.keyDown(editor, { key: 'b', ctrlKey: true });
  fireEvent.paste(editor, { clipboardData: { getData: () => 'Explain' } });
  expect(editor.querySelector('strong')).toHaveTextContent('Explain');
  fireEvent.keyDown(editor, { key: 'z', ctrlKey: true });
  expect(editor).not.toHaveTextContent('Explain');
  fireEvent.keyDown(editor, { key: 'z', ctrlKey: true, shiftKey: true });
  expect(editor).toHaveTextContent('Explain');
});

test('missing clipboard text changes nothing', () => {
  const onChange = vi.fn();
  render(<ProseEditor initialText="Keep these notes" onChange={onChange} />);
  fireEvent.paste(screen.getByRole('textbox'), {});
  expect(onChange).not.toHaveBeenCalled();
  expect(screen.getByRole('textbox')).toHaveTextContent('Keep these notes');
});
