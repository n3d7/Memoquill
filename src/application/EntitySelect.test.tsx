import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { expect, it } from 'vitest';
import { EntitySelect } from './EntitySelect';

it('reaches the final choice and preserves its identity across filtering and paging', async () => {
  const items = Object.fromEntries(Array.from({ length: 101 }, (_, index) => {
    const id = String(index);
    return [id, { id, title: `Lesson ${id}`, archived: index === 99 }];
  }));
  function Harness() {
    const [value, setValue] = useState('');
    return <EntitySelect label="Lesson" items={items} value={value} onChange={setValue} placeholder="Choose lesson" required />;
  }
  const user = userEvent.setup();
  render(<Harness />);
  const select = screen.getByRole('combobox', { name: 'Lesson' });
  expect(within(select).getAllByRole('option')).toHaveLength(41);
  const last = screen.getByRole('button', { name: 'Last choices' });
  last.focus();
  await user.keyboard('{Enter}');
  expect(screen.getByRole('status')).toHaveTextContent('Showing 81–100 of 100 choices.');
  await user.selectOptions(select, '100');
  await user.type(screen.getByRole('searchbox', { name: 'Filter lesson' }), 'Lesson 0');
  expect(select).toHaveValue('100');
  expect(within(select).getAllByRole('option').map(option => option.textContent))
    .toEqual(['Choose lesson', 'Lesson 100', 'Lesson 0']);
  expect(select).toBeRequired();
});
