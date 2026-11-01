import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import status from '../tests/fixtures/workspace-status.json';
import { App } from './App';
import type { Library } from './contentBridge';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

it('loads production workspace preferences independently of organization and preserves subject selection when saving them', async () => {
  const data: Library = { workspaceId: status.preferences.workspaceId, revision: 1, subjects: { quantum: { id: 'quantum', title: 'Quantum physics', priorKnowledge: null, archived: false } }, resources: {}, collections: {}, goals: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === 'workspace_status') return { ...status, schemaVersion: 4 };
    if (command === 'library') return data;
    if (command === 'save_reading_preferences') {
      const request = (args as { request: { preferences: typeof status.preferences.preferences } }).request;
      return { ...status.preferences, revision: 4, preferences: request.preferences };
    }
    throw new Error(`Unexpected mock command: ${command}`);
  });
  const user = userEvent.setup(); render(<App />);
  const subject = await screen.findByRole('button', { name: 'Quantum physics' }); await user.click(subject);
  await waitFor(() => expect(screen.getByRole('combobox', { name: 'Appearance' })).toHaveValue('dark'));
  fireEvent.change(screen.getByRole('combobox', { name: 'Appearance' }), { target: { value: 'light' } });
  await user.click(screen.getByRole('button', { name: 'Save reading preferences' }));
  expect(await screen.findByRole('dialog', { name: 'Operation result' })).toHaveTextContent('Reading preferences saved');
  expect(subject).toHaveAttribute('aria-current', 'page');
  expect(screen.getByRole('combobox', { name: 'Appearance' })).toHaveValue('light');
  expect(invoke).toHaveBeenCalledWith('save_reading_preferences', { request: expect.objectContaining({ workspaceId: data.workspaceId, expectedRevision: 3 }) });
});
