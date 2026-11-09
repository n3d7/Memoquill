import { beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeCoursePorts, nativeCoursePort } from './nativeCoursePort';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeEach(() => { vi.mocked(invoke).mockReset(); vi.mocked(invoke).mockResolvedValue(null); });
it('binds ingestion actions to their course', async () => {
  const a = createNativeCoursePorts('course-a');
  await a.ingestion!.list('resource', 0);
  await a.ingestion!.read('job');
  expect(vi.mocked(invoke).mock.calls).toEqual([
    ['ingestion_list', { courseId: 'course-a', resourceId: 'resource', offset: 0 }],
    ['ingestion_read', { courseId: 'course-a', jobId: 'job' }],
  ]);
});
it('binds every data-bearing operation to one immutable course, including raw lesson envelopes', async () => {
  const a = createNativeCoursePorts('course-a'); const b = createNativeCoursePorts('course-b');
  await a.content.library(); await b.content.library();
  expect(vi.mocked(invoke).mock.calls).toEqual([['library', { courseId: 'course-a' }], ['library', { courseId: 'course-b' }]]);
  vi.mocked(invoke).mockClear();
  const request = { contractVersion: 1 as const, workspaceId: 'w', operationId: 'o', expectedRevision: 0 };
  await a.content.searchResources('needle', 3); await a.content.applyContent({ ...request, command: { type: 'createResource', id: 'r', title: 'R' } });
  await a.content.readDocument('r'); await a.content.saveDocument({ ...request, resourceId: 'r', document: { schemaVersion: 1, blocks: [] } });
  await a.content.readingPosition('r'); await a.content.annotations('r');
  await a.content.saveReadingPosition({ ...request, position: { resourceId: 'r', revisionId: 'v', blockId: 'block', offsetUtf16: 0, revision: 0 } });
  await a.content.saveAnnotation({ ...request, annotation: { id: 'note', resourceId: 'r', revisionId: 'v', revision: 0, body: 'Note', anchor: { resourceId: 'r', revisionId: 'v', blockId: 'block', startUtf16: 0, endUtf16: 0, exactQuote: '', quoteHash: null } } });
  await a.sources.pickSource(request); await a.sources.sources('r'); await a.sources.sourcePreview('s');
  await a.sources.saveSource({ ...request, source: { id: 's', resourceId: 'r', digest: 'digest', filename: 'f', mediaType: 'text/plain', attribution: null, license: null, originUrl: null } });
  await a.workspace.workspaceStatus(); await a.workspace.reopenWorkspace();
  await a.workspace.saveReadingPreferences({ ...request, preferences: { theme: 'dark', textScale: 150 } });
  await a.lessons!.discover(); await a.lessons!.listDrafts('u'); await a.lessons!.read('u');
  await a.lessons!.readRevision('u', 'v'); await a.lessons!.readSource('u', 'v'); await a.lessons!.readResource('u', 'v'); await a.lessons!.readActivity('u', 'v');
  await a.lessons!.getReceipt('o');
  await a.lessons!.createDraft({ contractVersion: 1, workspaceId: 'w', unitId: 'u', operationId: 'o', expectedHeadRevision: null });
  const scope = { contractVersion: 1 as const, workspaceId: 'w', unitId: 'u', draftId: 'd', expectedDraftRevision: 0 };
  await a.lessons!.editDraft({ ...scope, operationId: 'o', operations: [] });
  await a.lessons!.validateDraft(scope); await a.lessons!.previewDraft(scope);
  await a.lessons!.applyDraft({ ...scope, operationId: 'o', expectedHeadRevision: null, previewHash: 'h' });
  await a.lessons!.discardDraft({ ...scope, operationId: 'o' });
  expect(vi.mocked(invoke).mock.calls).toHaveLength(29);
  for (const [, body] of vi.mocked(invoke).mock.calls) {
    const envelope = ArrayBuffer.isView(body) ? JSON.parse(new TextDecoder().decode(body)) : body;
    expect(envelope).toHaveProperty('courseId', 'course-a');
  }
});
it('catalogue operations carry metadata only and exact create retries preserve the operation identity', async () => {
  await nativeCoursePort.list(); await nativeCoursePort.create('op', '<b>Title</b>'); await nativeCoursePort.select('b');
  expect(vi.mocked(invoke).mock.calls).toEqual([['courses_list'], ['course_create', { request: { operationId: 'op', title: '<b>Title</b>' } }], ['course_select', { courseId: 'b' }]]);
});

it('unbound compatibility facades fail before invoking data commands', async () => {
  const content = await import('../contentBridge'); const sources = await import('../sourceBridge'); const workspace = await import('../workspaceBridge');
  await expect(content.library()).rejects.toThrow('course-bound');
  await expect(sources.sources('r')).rejects.toThrow('course-bound');
  await expect(workspace.workspaceStatus()).rejects.toThrow('course-bound');
  expect(invoke).not.toHaveBeenCalled();
});
