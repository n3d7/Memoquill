import { expect, it } from 'vitest';
import { createGuestIdentityPort } from './identity';

it('reopens the same local guest namespace and separates workspace and profile identity', () => {
  const guest = createGuestIdentityPort();
  expect(guest.session('workspace')).toEqual(createGuestIdentityPort().session('workspace'));
  expect(guest.session('workspace').actorId).not.toBe(guest.session('other').actorId);
  expect(guest.session('workspace').actorId).not.toBe(createGuestIdentityPort('other').session('workspace').actorId);
  expect(guest.session('workspace')).toEqual({ kind: 'localGuest', actorId: 'guest:local-default:workspace', profileId: 'local-default', workspaceId: 'workspace' });
});
it('keeps separator-bearing scope IDs distinct and rejects absent scope identity', () => {
  expect(createGuestIdentityPort('a:b').session('c').actorId).not.toBe(createGuestIdentityPort('a').session('b:c').actorId);
  expect(() => createGuestIdentityPort('')).toThrow('profile');
  expect(() => createGuestIdentityPort().session('')).toThrow('workspace');
});
