import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, test, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import initial from '../tests/fixtures/workspace-status.json';
import { App } from './App';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));
beforeEach(() => vi.resetAllMocks());

test('loads saved appearance, explicitly saves edits and restores them on reopening', async () => {
  let status = structuredClone(initial);
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === 'save_reading_preferences') {
      const { request } = args as { request: { preferences: typeof status.preferences.preferences } };
      status = { ...status, preferences: { ...status.preferences, revision: 4, preferences: request.preferences } };
      return status.preferences;
    }
    return status;
  });
  const view = render(<App />);
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Appearance' })).toHaveValue('dark'));
  expect(document.documentElement.style.fontSize).toBe('125%');
  fireEvent.change(screen.getByRole('combobox', { name: 'Appearance' }), { target: { value: 'light' } });
  const save = screen.getByRole('button', { name: 'Save reading preferences' });
  await userEvent.click(save);
  expect(await screen.findByRole('dialog', { name: 'Operation result' })).toHaveTextContent('Reading preferences saved');
  expect(invoke).toHaveBeenCalledWith('save_reading_preferences', { request: expect.objectContaining({
    contractVersion: 1, workspaceId: initial.preferences.workspaceId, expectedRevision: 3,
    operationId: expect.stringMatching(/^[a-f0-9-]{36}$/), preferences: { theme: 'light', textScale: 125 },
  }) });
  await userEvent.click(screen.getByRole('button', { name: 'Close result' }));
  expect(save).toHaveFocus();
  view.unmount();render(<App />);
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Appearance' })).toHaveValue('light'));
  expect(screen.getByText(/only until this window closes/)).toBeVisible();
});

test('unknown outcome retains one operation for retry and preserves material edits', async () => {
  const requests: unknown[] = [];
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === 'save_reading_preferences') {
      requests.push(args);
      if (requests.length === 1) throw { code: 'outcome_unconfirmed', path: '/private/do-not-display' };
      return { ...initial.preferences, revision: 4, preferences: { theme: 'light', textScale: 125 } };
    }
    return initial;
  });
  render(<App />);
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Appearance' })).toHaveValue('dark'));
  fireEvent.change(screen.getByRole('textbox', { name: 'Title' }), { target: { value: 'Preserve these notes' } });
  fireEvent.change(screen.getByRole('combobox', { name: 'Appearance' }), { target: { value: 'light' } });
  await userEvent.click(screen.getByRole('button', { name: 'Save reading preferences' }));
  expect(await screen.findByRole('dialog')).toHaveTextContent('could not be confirmed');
  expect(screen.queryByText('/private/do-not-display')).not.toBeInTheDocument();
  await userEvent.click(screen.getByRole('button', { name: 'Close result' }));
  await userEvent.click(screen.getByRole('button', { name: 'Retry saving preferences' }));
  expect(await screen.findByRole('dialog')).toHaveTextContent('Reading preferences saved. Material edits are still temporary.');
  expect(requests).toHaveLength(2);expect(requests[1]).toEqual(requests[0]);
  expect(invoke).toHaveBeenCalledWith('reopen_workspace');
  expect(screen.getByRole('textbox', { name: 'Title' })).toHaveValue('Preserve these notes');
});

test('startup failure is recoverable without clearing the material or exposing native details', async () => {
  vi.mocked(invoke).mockRejectedValueOnce({ code: 'unsupported_schema', privatePath: '/secret' }).mockResolvedValue(initial);
  render(<App />);
  expect(await screen.findByText(/workspace was created by an unsupported version/i)).toBeVisible();
  expect(screen.getByRole('textbox', { name: 'Learning notes' })).toHaveTextContent('Recall café 世界');
  await userEvent.click(screen.getByRole('button', { name: 'Retry workspace' }));
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Appearance' })).toHaveValue('dark'));
  expect(screen.queryByText('/secret')).not.toBeInTheDocument();
});
