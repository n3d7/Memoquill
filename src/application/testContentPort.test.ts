import { expect, it } from 'vitest';
import { createTestContentPort } from './testContentPort';
import type { Library, SaveDocument } from './contentTypes';

const library: Library = { workspaceId: 'workspace', revision: 0, resources: { resource: { id: 'resource', title: 'Notes', archived: false } }, subjects: {}, goals: {}, collections: {}, placements: {}, units: {}, paths: {}, relations: {}, tags: {} };
const request: SaveDocument = { contractVersion: 1, workspaceId: 'workspace', operationId: 'operation', expectedRevision: 0, resourceId: 'resource', document: { schemaVersion: 1, blocks: [{ id: 'block', content: { type: 'paragraph', inlines: [{ text: 'draft', marks: [] }] } }] } };
it('retains a deterministic durable save after a lost reply and rejects a changed retry', async () => {
  const adapter = createTestContentPort({ library });
  adapter.loseNextSaveReply();
  await expect(adapter.port.saveDocument(request)).rejects.toEqual({ code: 'outcome_unconfirmed' });
  const receipt = await adapter.port.saveDocument(request);
  expect(receipt).toMatchObject({ id: 'test:resource:1', revision: 1, document: request.document });
  await expect(adapter.port.saveDocument({ ...request, expectedRevision: 1 })).rejects.toEqual({ code: 'invalid_request' });
  expect(await adapter.port.readDocument('resource')).toEqual(receipt);
  expect(adapter.requests).toEqual([request, request, { ...request, expectedRevision: 1 }]);
});
it('keeps fixtures and returned snapshots independent and scopes writes to their workspace', async () => {
  const first = createTestContentPort({ library }); const second = createTestContentPort({ library });
  const snapshot = await first.port.library(); snapshot.resources.resource.title = 'Changed view';
  expect((await second.port.library()).resources.resource.title).toBe('Notes');
  expect((await first.port.library()).resources.resource.title).toBe('Notes');
  await expect(first.port.saveDocument({ ...request, workspaceId: 'other' })).rejects.toEqual({ code: 'permission_denied' });
  expect(await first.port.readDocument('resource')).toBeNull();
});
