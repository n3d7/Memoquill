import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, test, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import axe from 'axe-core';
import { App } from './App';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
beforeEach(() => vi.resetAllMocks());

test('validates edited material through the typed service and preserves Unicode', async () => {
  vi.mocked(invoke).mockResolvedValue({ title: '世界', proseCharacters: 6, codeLines: 1 });
  render(<App />);
  fireEvent.change(screen.getByRole('textbox', { name: 'Title' }), { target: { value: '世界' } });
  await userEvent.click(screen.getByRole('button', { name: 'Check draft' }));
  expect(invoke).toHaveBeenCalledWith('check_draft', { draft: expect.objectContaining({ schemaVersion: 1, title: '世界' }) });
  expect(await screen.findByRole('dialog', { name: 'Operation result' })).toHaveTextContent('Draft checked: 世界');
  const close = screen.getByRole('button', { name: 'Close result' });
  expect(close).toHaveFocus();
  await userEvent.tab({ shift: true });
  expect(close).toHaveFocus();
  await userEvent.click(close);
  expect(screen.getByRole('button', { name: 'Check draft' })).toHaveFocus();
  expect(screen.getByText(/only until this window closes/)).toBeVisible();
});

test('reports service failure without clearing edits', async () => {
  vi.mocked(invoke).mockRejectedValue('Unsupported draft schema.');
  render(<App />);
  await userEvent.click(screen.getByRole('button', { name: 'Check draft' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Unsupported draft schema.');
  expect(screen.getByRole('dialog', { name: 'Operation result' })).toHaveAccessibleDescription('Unsupported draft schema.');
  expect(screen.getByRole('textbox', { name: 'Title' })).not.toHaveValue('');
});

test('unexpected failures show a safe message instead of raw object data', async () => {
  vi.mocked(invoke).mockRejectedValue({ privateDetail: 'do-not-display' });
  render(<App />);
  await userEvent.click(screen.getByRole('button', { name: 'Check draft' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Your edits are still here');
  expect(screen.queryByText('do-not-display')).not.toBeInTheDocument();
});

test('appearance and text size can change without resetting notes', () => {
  const { container } = render(<App />);
  fireEvent.change(screen.getByRole('combobox', { name: 'Appearance' }), { target: { value: 'dark' } });
  expect(container.querySelector('.app')).toHaveAttribute('data-theme', 'dark');
  fireEvent.change(screen.getByRole('combobox', { name: 'Text size' }), { target: { value: '200' } });
  expect(document.documentElement.style.fontSize).toBe('200%');
  expect(screen.getByRole('textbox', { name: 'Learning notes' })).toHaveTextContent('Recall café 世界');
  document.documentElement.style.fontSize = '';
});

test('opens a preview and restores keyboard focus to its trigger', async () => {
  render(<App />);
  const trigger = screen.getByRole('button', { name: 'Read preview' });
  await userEvent.click(trigger);
  expect(screen.getByRole('dialog', { name: 'Reading preview' })).toBeVisible();
  expect(screen.getByRole('button', { name: 'Close preview' })).toHaveFocus();
  await userEvent.tab();
  expect(screen.getByRole('button', { name: 'Close preview' })).toHaveFocus();
  expect(fireEvent.keyDown(screen.getByRole('dialog'), { key: 'PageDown' })).toBe(true);
  expect(screen.getByRole('dialog')).toBeVisible();
  await userEvent.click(screen.getByRole('button', { name: 'Close preview' }));
  expect(trigger).toHaveFocus();
});

test('native picker cancellation and failure preserve the fixture', async () => {
  vi.mocked(open).mockResolvedValueOnce(null).mockRejectedValueOnce(new Error('Picker unavailable'));
  render(<App />);
  const trigger = screen.getByRole('button', { name: 'Choose a source file' });
  await userEvent.click(trigger);
  expect(screen.getByRole('status')).toHaveTextContent('No file selected');
  const details = screen.getByRole('button', { name: 'Read last result' });
  await userEvent.click(details);
  expect(screen.getByRole('dialog', { name: 'Operation result' })).toHaveAccessibleDescription('No file selected. Your edits are unchanged.');
  await userEvent.click(screen.getByRole('button', { name: 'Close result' }));
  expect(details).toHaveFocus();
  await userEvent.click(trigger);
  expect(await screen.findByRole('alert')).toHaveTextContent('Picker unavailable');
  expect(trigger).toHaveFocus();
});

test('selecting a source discloses only its name and does not import or execute it', async () => {
  vi.mocked(open).mockResolvedValue('/private/folder/lesson.py');
  render(<App />);
  await userEvent.click(screen.getByRole('button', { name: 'Choose a source file' }));
  expect(screen.getByRole('status')).toHaveTextContent('Selected lesson.py');
  expect(screen.queryByText('/private/folder/lesson.py')).not.toBeInTheDocument();
  expect(invoke).not.toHaveBeenCalled();
});

test('supports readable math and passive code without active links', async () => {
  render(<App />);
  fireEvent.change(screen.getByRole('textbox', { name: 'Formula source (LaTeX)' }), { target: { value: String.raw`\href{https://example.com}{x}` } });
  await userEvent.click(screen.getByRole('button', { name: 'Read preview' }));
  expect(screen.getByRole('dialog').querySelector('math')).not.toBeNull();
  expect(screen.getByRole('dialog').querySelector('a')).toBeNull();
  expect(screen.getByRole('dialog').querySelector('pre code')).toHaveTextContent('return');
});

test('exposes labelled controls with no automated accessibility violations', async () => {
  const { container } = render(<App />);
  await waitFor(() => expect(screen.getByRole('textbox', { name: 'Learning notes' })).toBeInTheDocument());
  const result = await axe.run(container, { rules: { 'color-contrast': { enabled: false } } });
  expect(result.violations).toEqual([]);
});
