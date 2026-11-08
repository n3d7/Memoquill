import type { Library, SaveDocument, SavedDocument } from './contentTypes';
import type { ContentPort } from './ports';

export interface TestContentFixture { library: Library; documents?: SavedDocument[] }
const unavailable = async (): Promise<never> => { throw { code: 'test_capability_unavailable' }; };

/** Explicit offline test substitute. Unsupported scenarios fail instead of invoking IPC. */
export function createTestContentPort(fixture: TestContentFixture) {
  const snapshot = structuredClone(fixture.library);
  const documents = new Map((fixture.documents ?? []).map(value => [value.resourceId, structuredClone(value)]));
  const revisions = new Map((fixture.documents ?? []).map(value => [value.id, structuredClone(value)]));
  const receipts = new Map<string, { request: string; saved: SavedDocument }>();
  let loseReply = false;
  const requests: SaveDocument[] = [];
  const port: ContentPort = { library: async () => structuredClone(snapshot), searchResources: unavailable, applyContent: unavailable,
    readDocument: async (resourceId, revisionId) => {
      const value = revisionId ? revisions.get(revisionId) : documents.get(resourceId);
      return value?.resourceId === resourceId ? structuredClone(value) : null;
    },
    saveDocument: async request => {
      requests.push(structuredClone(request));
      if (request.workspaceId !== snapshot.workspaceId) throw { code: 'permission_denied' };
      const payload = JSON.stringify(request); const prior = receipts.get(request.operationId);
      if (prior) {
        if (prior.request !== payload) throw { code: 'invalid_request' };
        return structuredClone(prior.saved);
      }
      if (!snapshot.resources[request.resourceId]) throw { code: 'invalid_request' };
      const revision = documents.get(request.resourceId)?.revision ?? 0;
      if (request.expectedRevision !== revision) throw { code: 'revision_conflict' };
      const saved: SavedDocument = { id: `test:${request.resourceId}:${revision + 1}`, resourceId: request.resourceId, revision: revision + 1, document: structuredClone(request.document) };
      documents.set(saved.resourceId, saved); revisions.set(saved.id, saved);
      receipts.set(request.operationId, { request: payload, saved });
      if (loseReply) { loseReply = false; throw { code: 'outcome_unconfirmed' }; }
      return structuredClone(saved);
    }, readingPosition: unavailable,
    saveReadingPosition: unavailable, annotations: unavailable, saveAnnotation: unavailable };
  return { port, requests, loseNextSaveReply() { loseReply = true; } };
}
