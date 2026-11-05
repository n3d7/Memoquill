import { expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { LibraryApp } from '../LibraryApp';
import { ApplicationProvider, useIdentityPort, useLessonPort } from './ApplicationProvider';
import { createGuestIdentityPort } from './identity';
import { createTestContentPort } from './testContentPort';
import type { Library } from './contentTypes';
import type { SavePreferences, WorkspaceStatus } from './workspaceTypes';
import type { LessonPort } from '../lessons/types';

const native = vi.hoisted(() => vi.fn().mockRejectedValue(new Error('No native transport')));
vi.mock('@tauri-apps/api/core', () => ({ invoke: native }));
const library: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Injected notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
const status: WorkspaceStatus = { contractVersion: 1, schemaVersion: 5, preferences: { workspaceId: 'workspace', revision: 0, preferences: { theme: 'system', textScale: 100 } }, sqliteVersion: 'test', recovery: { interruptedOperations: 0, stagingFiles: 0, orphanAssets: 0, missingAssets: 0, corruptAssets: 0, unsafePaths: 0 } };
it('runs the existing workspace screen with injected application services and a working local guest', async () => {
  const saves: SavePreferences[] = [];
  const workspace = { workspaceStatus: async () => structuredClone(status), reopenWorkspace: async () => structuredClone(status), saveReadingPreferences: async (request: SavePreferences) => { saves.push(request); return { workspaceId: 'workspace', revision: 1, preferences: request.preferences }; } };
  render(<ApplicationProvider ports={{ content: createTestContentPort({ library }).port, workspace }}><LibraryApp /></ApplicationProvider>);
  await screen.findByRole('button', { name: 'Injected notes' });
  expect(screen.getByLabelText('Local identity')).toHaveTextContent('Local guest · Offline workspace');
  const user = userEvent.setup();
  await user.selectOptions(screen.getByLabelText('Appearance'), 'dark');
  await user.click(screen.getByRole('button', { name: 'Save reading preferences' }));
  expect(saves).toHaveLength(1);
  expect(saves[0]).toMatchObject({ workspaceId: 'workspace', expectedRevision: 0, preferences: { theme: 'dark', textScale: 100 } });
  expect(saves[0]).not.toHaveProperty('actorId');
  expect(native).not.toHaveBeenCalled();
});
it('keeps lesson service and local identity separate from native transport and UI authority', async () => {
  const read = vi.fn().mockResolvedValue(null);
  const unsupported = async (): Promise<never> => { throw new Error('Unused lesson capability'); };
  const lessons: LessonPort = { read, discover: unsupported, listDrafts: unsupported, createDraft: unsupported, editDraft: unsupported, validateDraft: unsupported, previewDraft: unsupported, applyDraft: unsupported, discardDraft: unsupported, getReceipt: unsupported, readRevision: unsupported, readSource: unsupported, readResource: unsupported, readActivity: unsupported };
  function Consumer() { const lesson = useLessonPort(); const guest = useIdentityPort().session('workspace'); return <button onClick={() => void lesson.read('unit')}>{guest.kind}</button>; }
  render(<ApplicationProvider ports={{ lessons, identity: createGuestIdentityPort('profile') }}><Consumer /></ApplicationProvider>);
  await userEvent.setup().click(screen.getByRole('button', { name: 'localGuest' }));
  expect(read).toHaveBeenCalledExactlyOnceWith('unit');
  expect(native).not.toHaveBeenCalled();
});
