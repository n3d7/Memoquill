import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createNativeLessonPort } from './nativeLessonPort';
import type { SavedDocument } from '../application/contentTypes';
import type { CreateLessonDraft, EditLessonDraft, LessonError } from './types';
const nativeLessonPort = createNativeLessonPort('course');
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
describe('lesson native transport', () => {
  beforeEach(() => { vi.mocked(invoke).mockReset(); });
  it('sends the versioned envelope as UTF-8 raw bytes', async () => {
    vi.mocked(invoke).mockResolvedValue(null);
    const request: CreateLessonDraft = { contractVersion: 1, workspaceId: 'workspace', unitId: 'unit', operationId: 'operation', expectedHeadRevision: null };
    await nativeLessonPort.createDraft(request);
    const [command, body] = vi.mocked(invoke).mock.calls[0];
    expect(command).toBe('lesson_create_draft');
    expect(ArrayBuffer.isView(body)).toBe(true);
    if (!ArrayBuffer.isView(body)) throw new Error('Expected native raw byte envelope');
    expect(JSON.parse(new TextDecoder().decode(body))).toEqual({ courseId: 'course', request });
  });
  it('preserves native safe field diagnostics on rejection', async () => {
    const error: LessonError = { code: 'invalid_request', diagnostics: [{ code: 'invalid_props', nodeId: 'node', operationIndex: 0, field: 'props.gap', repair: 'use_declared_value' }] };
    vi.mocked(invoke).mockRejectedValue(error);
    const request: EditLessonDraft = { contractVersion: 1, workspaceId: 'workspace', unitId: 'unit', draftId: 'draft', expectedDraftRevision: 1, operationId: 'operation', operations: [] };
    let caught: unknown;
    try { await nativeLessonPort.editDraft(request); } catch (failure) { caught = failure; }
    expect(caught).toEqual(error);
  });
});

it('reads a resource pin with its Unit scope and preserves null denial', async () => {
  vi.mocked(invoke).mockResolvedValue(null);
  expect(await nativeLessonPort.readResource('unit', 'pin')).toBeNull();
  expect(invoke).toHaveBeenCalledWith('lesson_read_resource', { courseId: 'course', unitId: 'unit', revisionId: 'pin' });
});

it('returns the immutable resource snapshot without transforming it', async () => {
  const saved: SavedDocument = { id: 'pin', resourceId: 'resource', revision: 1, document: { schemaVersion: 1, blocks: [] } };
  vi.mocked(invoke).mockResolvedValue(saved);
  expect(await nativeLessonPort.readResource('unit', 'pin')).toBe(saved);
});
