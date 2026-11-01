import { fireEvent, render, screen, within } from '@testing-library/react';
import { expect, test, vi } from 'vitest';
import { DocumentEditor } from './DocumentEditor';
import type { Document } from './documentModel';

const initial = (): Document => ({ schemaVersion: 1, blocks: [{ id: 'p', content: { type: 'paragraph', inlines: [{ text: 'Hello 世界', marks: [] }] } }] });
const paste = (editor: HTMLElement, text: string) => fireEvent.paste(editor, { clipboardData: { getData: (type: string) => type === 'text/plain' ? text : '<img src="https://example.test/secret" onerror="alert(1)">' } });

test('renaming a nested editor preserves its view, focus and undo history', () => {
  const changed = vi.fn();
  const { rerender } = render(<DocumentEditor document={initial()} label="List item 2" onChange={changed} />);
  const editor = screen.getByRole('textbox', { name: 'List item 2' });
  editor.focus();
  paste(editor, 'Added ');
  rerender(<DocumentEditor document={changed.mock.lastCall![0]} label="List item 1" onChange={changed} />);
  expect(screen.getByRole('textbox', { name: 'List item 1' })).toBe(editor);
  expect(editor).toHaveFocus();
  fireEvent.keyDown(editor, { key: 'z', ctrlKey: true });
  expect(editor).toHaveTextContent('Hello 世界');
  expect(editor).not.toHaveTextContent('Added');
});

test('explicit resume selects the identified block at a bounded UTF16 caret without rebuilding the view', () => {
  const changed = vi.fn();
  const value: Document = { schemaVersion: 1, blocks: [...initial().blocks,
    { id: 'target', content: { type: 'paragraph', inlines: [{ text: 'a😀b', marks: [] }] } },
    { id: 'formula', content: { type: 'math', source: 'x' } },
  ] };
  const { rerender } = render(<DocumentEditor document={value} onChange={changed} />);
  const editor = screen.getByRole('textbox', { name: 'Learning document' });
  const resume = { blockId: 'target', offsetUtf16: 3 };
  rerender(<DocumentEditor document={value} onChange={changed} resume={resume} />);
  expect(screen.getByLabelText('Selected block')).toHaveValue('target');
  expect(screen.getByRole('textbox', { name: 'Learning document' })).toBe(editor);
  expect(editor).toHaveFocus();
  expect(window.getSelection()!.anchorNode?.textContent).toBe('a😀b');
  expect(window.getSelection()!.anchorOffset).toBe(3);
  paste(editor, '!');
  expect(editor.querySelectorAll('p')[1]).toHaveTextContent('a😀!b');
  rerender(<DocumentEditor document={changed.mock.lastCall![0]} onChange={vi.fn()} resume={resume} />);
  expect(window.getSelection()!.anchorOffset).toBe(4);
  rerender(<DocumentEditor document={changed.mock.lastCall![0]} onChange={changed} resume={{ blockId: 'target', offsetUtf16: 999 }} />);
  expect(window.getSelection()!.anchorOffset).toBe(5);
  rerender(<DocumentEditor document={changed.mock.lastCall![0]} onChange={changed} resume={{ blockId: 'formula', offsetUtf16: 99 }} />);
  expect(screen.getByLabelText('Selected block')).toHaveValue('formula');
  expect(editor.querySelector('.ProseMirror-selectednode')).toHaveTextContent('Formula: x');
  expect(changed).toHaveBeenCalledTimes(1);
});

test('keeps the editor DOM, selection and undo history through callback and save-status rerenders', () => {
  const first = vi.fn();
  const { rerender } = render(<DocumentEditor document={initial()} onChange={first} />);
  const editor = screen.getByRole('textbox', { name: 'Learning document' });
  fireEvent.keyDown(editor, { key: 'ArrowRight' });
  paste(editor, 'A');
  expect(first).toHaveBeenCalled();
  const next = vi.fn();
  rerender(<DocumentEditor document={first.mock.lastCall![0]} onChange={next} />);
  expect(screen.getByRole('textbox', { name: 'Learning document' })).toBe(editor);
  paste(editor, 'B');
  expect(editor).toHaveTextContent('ABHello 世界');
  expect(next).toHaveBeenCalled();
  fireEvent.keyDown(editor, { key: 'z', ctrlKey: true });
  expect(editor).toHaveTextContent('Hello 世界');
  expect(editor).not.toHaveTextContent('AB');
});

test('only reads plain clipboard text and blocks dropped HTML without active DOM', () => {
  const changed = vi.fn();
  render(<DocumentEditor document={initial()} onChange={changed} />);
  const editor = screen.getByRole('textbox', { name: 'Learning document' });
  const getData = vi.fn((type: string) => type === 'text/plain' ? '<script>literal</script> café' : '<img src="https://example.test/secret">');
  fireEvent.paste(editor, { clipboardData: { getData } });
  expect(getData).toHaveBeenCalledExactlyOnceWith('text/plain');
  expect(editor).toHaveTextContent('<script>literal</script> café');
  expect(editor.querySelector('script,img,a,iframe')).toBeNull();
  const before = editor.textContent;
  fireEvent.drop(editor, { dataTransfer: { getData: () => '<img src="https://example.test/drop">', files: [] } });
  expect(editor.textContent).toBe(before);
});

test('repairs split block identities and preserves code, math, nested text and captions', () => {
  const changed = vi.fn();
  const document: Document = { schemaVersion: 1, blocks: [...initial().blocks,
    { id: 'code', content: { type: 'code', text: 'x = "😀"\n', language: 'python' } },
    { id: 'math', content: { type: 'math', source: '\\alpha^2' } },
    { id: 'table', content: { type: 'table', rows: [[[{ id: 'cell', content: { type: 'paragraph', inlines: [{ text: 'Cell', marks: [{ type: 'strong' }] }] } }]]] } },
    { id: 'media', content: { type: 'media', assetDigest: 'a'.repeat(64), mime: 'image/png', caption: 'caption', alt: 'alt' } },
    { id: 'unsupported', content: { type: 'unsupported', original: '<script>keep</script>', reason: 'Unknown source' } },
  ] };
  render(<DocumentEditor document={document} onChange={changed} />);
  const editor = screen.getByRole('textbox', { name: 'Learning document' });
  fireEvent.keyDown(editor, { key: 'Enter' });
  let value: Document = changed.mock.lastCall![0];
  expect(new Set(value.blocks.map(block => block.id)).size).toBe(value.blocks.length);
  fireEvent.change(screen.getByLabelText('Selected block'), { target: { value: 'math' } });
  fireEvent.change(screen.getByLabelText('Formula source'), { target: { value: '\\beta + 世界' } });
  value = changed.mock.lastCall![0];
  expect(value.blocks.find(block => block.id === 'math')?.content).toEqual({ type: 'math', source: '\\beta + 世界' });
  fireEvent.change(screen.getByLabelText('Selected block'), { target: { value: 'table' } });
  const nested = screen.getByRole('textbox', { name: 'Table row 1 cell 1' });
  paste(nested, 'é');
  value = changed.mock.lastCall![0];
  const table = value.blocks.find(block => block.id === 'table')!.content;
  expect(table.type === 'table' && table.rows[0][0][0].id).toBe('cell');
  expect(within(nested).getByText('éCell').tagName).toBe('STRONG');
  fireEvent.change(screen.getByLabelText('Selected block'), { target: { value: 'media' } });
  fireEvent.change(screen.getByLabelText('Media caption'), { target: { value: 'new caption 世界' } });
  value = changed.mock.lastCall![0];
  expect(value.blocks.find(block => block.id === 'code')?.content).toEqual(document.blocks[1].content);
  expect(value.blocks.find(block => block.id === 'media')?.content).toMatchObject({ caption: 'new caption 世界', alt: 'alt', assetDigest: 'a'.repeat(64) });
  expect(value.blocks.find(block => block.id === 'unsupported')?.content).toEqual(document.blocks[5].content);
});

test('read-only blocks paste, formatting and structured edits', () => {
  const changed = vi.fn();
  render(<DocumentEditor document={initial()} onChange={changed} readOnly />);
  paste(screen.getByRole('textbox', { name: 'Learning document' }), 'No');
  expect(changed).not.toHaveBeenCalled();
  expect(screen.getByRole('button', { name: 'Bold' })).toBeDisabled();
});

test('does not replace a composing DOM or its text selection on an unrelated rerender', () => {
  const documentValue = initial();
  const { rerender } = render(<DocumentEditor document={documentValue} onChange={vi.fn()} />);
  const editor = screen.getByRole('textbox', { name: 'Learning document' });
  editor.focus();
  const text = editor.querySelector('p')!.firstChild!;
  const selection = window.getSelection()!;
  const range = document.createRange();
  range.setStart(text, 3); range.collapse(true);
  selection.removeAllRanges(); selection.addRange(range);
  fireEvent.compositionStart(editor, { data: '世' });
  rerender(<DocumentEditor document={{ ...documentValue }} onChange={vi.fn()} />);
  expect(screen.getByRole('textbox', { name: 'Learning document' })).toBe(editor);
  expect(editor.querySelector('p')!.firstChild).toBe(text);
  expect(selection.anchorNode).toBe(text);
  expect(selection.anchorOffset).toBe(3);
  fireEvent.compositionEnd(editor, { data: '世' });
});

test('adds and removes structured blocks using native keyboard accessible controls', () => {
  const changed = vi.fn();
  render(<DocumentEditor document={initial()} onChange={changed} />);
  fireEvent.change(screen.getByLabelText('New block type'), { target: { value: 'list' } });
  fireEvent.click(screen.getByRole('button', { name: 'Add block' }));
  expect(screen.getByRole('textbox', { name: 'List item 1' })).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Add list item' }));
  expect(screen.getByRole('textbox', { name: 'List item 2' })).toBeInTheDocument();
  const documentValue: Document = changed.mock.lastCall![0];
  const ids: string[] = [];
  for (const block of documentValue.blocks) {
    ids.push(block.id);
    if (block.content.type === 'list') ids.push(...block.content.items.flat().map(item => item.id));
  }
  expect(new Set(ids).size).toBe(ids.length);
  fireEvent.click(screen.getAllByRole('button', { name: 'Remove selected block' })[0]);
  expect(screen.queryByRole('textbox', { name: 'List item 1' })).toBeNull();
  expect((changed.mock.lastCall![0] as Document).blocks.map(block => block.id)).toEqual(['p']);
});

test.each(['list', 'table'] as const)('read-only %s blocks reject nested paste, drop and metadata changes, then unlock without rebuilding', type => {
  const changed = vi.fn();
  const content = type === 'list'
    ? { type, ordered: false, items: [initial().blocks] }
    : { type, rows: [[initial().blocks]] };
  const value: Document = { schemaVersion: 1, blocks: [{ id: 'outer', content }] };
  const { rerender } = render(<DocumentEditor document={value} onChange={changed} readOnly />);
  const label = type === 'list' ? 'List item 1' : 'Table row 1 cell 1';
  const nested = screen.getByRole('textbox', { name: label });
  expect(nested).toHaveAttribute('contenteditable', 'false');
  const getData = vi.fn(() => 'forbidden');
  expect(fireEvent.paste(nested, { clipboardData: { getData } })).toBe(false);
  expect(getData).not.toHaveBeenCalled();
  expect(fireEvent.drop(nested)).toBe(false);
  expect(fireEvent.dragStart(nested)).toBe(false);
  fireEvent.keyDown(nested, { key: 'Enter' });
  for (const button of screen.getAllByRole('button')) expect(button).toBeDisabled();
  expect(changed).not.toHaveBeenCalled();
  expect(nested).toHaveTextContent('Hello 世界');
  rerender(<DocumentEditor document={value} onChange={changed} />);
  expect(screen.getByRole('textbox', { name: label })).toBe(nested);
  expect(nested).toHaveAttribute('contenteditable', 'true');
  paste(nested, 'editable ');
  expect(changed).toHaveBeenCalledOnce();
  expect(nested).toHaveTextContent('editable Hello 世界');
});

test('plain multiline paste preserves empty paragraphs, while code keeps literal newlines and inert source', () => {
  const changed = vi.fn();
  render(<DocumentEditor document={initial()} onChange={changed} />);
  const editor = screen.getByRole('textbox', { name: 'Learning document' });
  paste(editor, 'first\r\n\r\nlast\r');
  expect(Array.from(editor.querySelectorAll('p'), paragraph => paragraph.textContent)).toEqual(['first', '', 'last', 'Hello 世界']);
  fireEvent.change(screen.getByLabelText('New block type'), { target: { value: 'code' } });
  fireEvent.click(screen.getByRole('button', { name: 'Add block' }));
  fireEvent.change(screen.getByLabelText('Code language'), { target: { value: 'html' } });
  paste(editor, '<script>literal</script>\n<img src="https://example.test/private">');
  expect(editor.querySelector('pre code')).toHaveTextContent('<script>literal</script>');
  expect(editor.querySelector('script,img,iframe')).toBeNull();
  const saved: Document = changed.mock.lastCall![0];
  expect(saved.blocks.at(-1)?.content).toEqual({ type: 'code', language: 'html', text: '<script>literal</script>\n<img src="https://example.test/private">' });
});

test('table row and column controls preserve rectangular content and retain the minimum cell', () => {
  const changed = vi.fn();
  render(<DocumentEditor document={initial()} onChange={changed} />);
  fireEvent.change(screen.getByLabelText('New block type'), { target: { value: 'table' } });
  fireEvent.click(screen.getByRole('button', { name: 'Add block' }));
  expect(screen.getByRole('button', { name: 'Remove last table column' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Remove table row 1' })).toBeDisabled();
  fireEvent.click(screen.getByRole('button', { name: 'Add table column' }));
  fireEvent.click(screen.getByRole('button', { name: 'Add table row' }));
  const cell = screen.getByRole('textbox', { name: 'Table row 2 cell 2' });
  paste(cell, 'retained');
  fireEvent.click(screen.getByRole('button', { name: 'Remove table row 1' }));
  expect(screen.getByRole('textbox', { name: 'Table row 1 cell 2' })).toBe(cell);
  expect(cell).toHaveTextContent('retained');
  fireEvent.click(screen.getByRole('button', { name: 'Remove last table column' }));
  const value: Document = changed.mock.lastCall![0];
  const table = value.blocks.at(-1)!.content;
  expect(table.type === 'table' && table.rows.map(row => row.length)).toEqual([1]);
  expect(screen.getByRole('button', { name: 'Remove last table column' })).toBeDisabled();
});

test('heading, media and unsupported source controls preserve passive payloads', () => {
  const changed = vi.fn();
  const value: Document = { schemaVersion: 1, blocks: [
    { id: 'h', content: { type: 'heading', level: 2, inlines: [{ text: 'Title', marks: [] }] } },
    { id: 'm', content: { type: 'media', assetDigest: 'a'.repeat(64), mime: 'text/html', caption: 'caption', alt: 'alt' } },
    { id: 'u', content: { type: 'unsupported', original: '<img src="https://example.test/source">', reason: 'source only', source: { assetDigest: 'b'.repeat(64), locator: '<script>location</script>' } } },
  ] };
  render(<DocumentEditor document={value} onChange={changed} />);
  fireEvent.change(screen.getByLabelText('Heading level'), { target: { value: '6' } });
  expect(screen.getByRole('heading', { name: 'Title', level: 6 })).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Selected block'), { target: { value: 'm' } });
  fireEvent.change(screen.getByLabelText('Media alternative text'), { target: { value: '<img>literal' } });
  fireEvent.change(screen.getByLabelText('Selected block'), { target: { value: 'u' } });
  const note = screen.getByRole('note');
  expect(note).toHaveTextContent('<img src="https://example.test/source">');
  expect(note).toHaveTextContent('<script>location</script>');
  expect(note.querySelector('img,script')).toBeNull();
  expect(changed.mock.lastCall![0].blocks[2]).toEqual(value.blocks[2]);
});
