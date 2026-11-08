import { render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';
import { DocumentEditor } from '../DocumentEditor';
it('reads canonical lists and tables in semantic order without authoring controls', () => {
  const paragraph = (id: string, text: string) => ({ id, content: { type: 'paragraph' as const, inlines: [{ text, marks: [] }] } });
  render(<DocumentEditor readOnly passive label="Lesson reading" onChange={() => {}} document={{ schemaVersion: 1, blocks: [paragraph('before', 'Before table'), { id: 'table', content: { type: 'table', rows: [[[paragraph('cell', 'Table cell')]]] } }, { id: 'list', content: { type: 'list', ordered: true, items: [[paragraph('item', 'List item')]] } }, paragraph('after', 'After list')] }} />);
  expect(screen.getByRole('table')).toHaveTextContent('Table cell'); expect(screen.getByRole('list')).toHaveTextContent('List item');
  expect(screen.getByText('Before table').compareDocumentPosition(screen.getByRole('table')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(screen.queryByRole('button')).not.toBeInTheDocument(); expect(screen.queryByText(/Ctrl/)).not.toBeInTheDocument();
});
