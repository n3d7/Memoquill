export interface GuestSession {
  kind: 'localGuest';
  actorId: string;
  profileId: string;
  workspaceId: string;
}
export interface IdentityPort { session(workspaceId: string): GuestSession }

/** A local namespace, never a grant or a trust assertion sent to storage. */
export function createGuestIdentityPort(profileId = 'local-default'): IdentityPort {
  if (!profileId) throw new Error('A local profile identity is required');
  return { session(workspaceId) {
    if (!workspaceId) throw new Error('A workspace identity is required');
    return { kind: 'localGuest', actorId: `guest:${encodeURIComponent(profileId)}:${encodeURIComponent(workspaceId)}`, profileId, workspaceId };
  } };
}
export const localGuestIdentity = createGuestIdentityPort();
