import type { ApplyLessonDraft, Composition, LessonDraft, LessonOperation, LessonPort, LessonPreview, LessonReceipt, RegistryDescriptor, SavedComposition } from './types';

export interface LessonState {
  composition: Composition | null; draft: LessonDraft | null; drafts: LessonDraft[];
  saved: SavedComposition | null; preview: LessonPreview | null; registry: RegistryDescriptor[];
  busy: boolean; pending: boolean; message: string; error: string; canUndo: boolean; canRedo: boolean;
}
interface History { before: Composition; after: Composition }
const message = (error: unknown) => error instanceof Error ? error.message : typeof error === 'object' && error && 'code' in error ? String(error.code) : 'The lesson change could not be confirmed.';
/** One durable authoring scope per Unit. Unknown outcomes retain the exact request. */
export class LessonController {
  private state: LessonState = { composition: null, draft: null, drafts: [], saved: null, preview: null, registry: [], busy: false, pending: false, message: '', error: '', canUndo: false, canRedo: false };
  private listeners = new Set<() => void>();
  private pending: (() => Promise<void>) | null = null;
  private undoStack: History[] = []; private redoStack: History[] = []; private generation = 0;
  constructor(readonly workspaceId: string, readonly unitId: string, readonly port: LessonPort) {}
  getSnapshot = () => this.state;
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  private publish(patch: Partial<LessonState>) { this.state = { ...this.state, ...patch, canUndo: this.undoStack.length > 0, canRedo: this.redoStack.length > 0 }; this.listeners.forEach(listener => listener()); }
  private scope() { return { contractVersion: 1 as const, workspaceId: this.workspaceId, unitId: this.unitId }; }
  private draftScope() { const d = this.state.draft; if (!d) throw new Error('Start an editable lesson first.'); return { ...this.scope(), draftId: d.id, expectedDraftRevision: d.revision }; }
  async load() {
    if (this.state.pending || this.state.busy) return;
    const generation = ++this.generation; this.publish({ busy: true, error: '' });
    try {
      const [saved, drafts, registry] = await Promise.all([this.port.read(this.unitId), this.port.listDrafts(this.unitId), this.port.discover()]);
      if (generation !== this.generation) return;
      const draft = drafts[0] ?? null; this.undoStack = []; this.redoStack = [];
      this.publish({ saved, drafts, draft, registry, composition: draft?.composition ?? saved?.composition ?? null, preview: null, message: draft ? 'Resumed your saved editing draft.' : '' });
    } catch (error) { this.publish({ error: message(error) }); } finally { if (generation === this.generation) this.publish({ busy: false }); }
  }
  async chooseDraft(id: string) { if (this.state.pending || this.state.busy) return; const draft = this.state.drafts.find(d => d.id === id); if (!draft) return; this.undoStack = []; this.redoStack = []; this.publish({ draft, composition: draft.composition, preview: null, error: '' }); }
  private async execute(work: () => Promise<void>) {
    this.pending = work; this.publish({ pending: true, busy: true, error: '' });
    try { await work(); this.pending = null; this.publish({ pending: false }); }
    catch (error) { this.publish({ error: `${message(error)} Your draft is preserved. Retry the same change or check for a conflict.`, message: '' }); }
    finally { this.publish({ busy: false }); }
  }
  async retry() { if (!this.pending || this.state.busy) return; await this.execute(this.pending); }
  async begin() {
    if (this.state.pending || this.state.busy || this.state.draft) return;
    const request = { ...this.scope(), operationId: crypto.randomUUID(), expectedHeadRevision: this.state.saved?.id ?? null };
    await this.execute(async () => { const draft = await this.port.createDraft(request); this.publish({ draft, composition: draft.composition, drafts: [...this.state.drafts, draft], preview: null }); });
  }
  async edit(operations: LessonOperation[], history: 'normal' | 'undo' | 'redo' = 'normal') {
    if (this.state.pending || this.state.busy || !this.state.draft || !this.state.composition) return;
    if (operations.length > 64) { this.publish({ error: 'This change needs more operations than one atomic edit supports. The draft and undo history are preserved.' }); return; }
    const before = structuredClone(this.state.composition);
    const request = { ...this.draftScope(), operationId: crypto.randomUUID(), operations: structuredClone(operations) };
    await this.execute(async () => {
      const draft = await this.port.editDraft(request);
      if (history === 'normal') { this.undoStack.push({ before, after: structuredClone(draft.composition) }); this.redoStack = []; }
      if (history === 'undo') { const entry = this.undoStack.pop(); if (entry) this.redoStack.push(entry); }
      if (history === 'redo') { const entry = this.redoStack.pop(); if (entry) this.undoStack.push(entry); }
      this.publish({ draft, composition: draft.composition, preview: null, message: 'Editing draft saved locally.' });
    });
  }
  async undo() { const entry = this.undoStack.at(-1); if (entry && this.state.composition) await this.edit(restoreOperations(this.state.composition, entry.before), 'undo'); }
  async redo() { const entry = this.redoStack.at(-1); if (entry && this.state.composition) await this.edit(restoreOperations(this.state.composition, entry.after), 'redo'); }
  async preview() {
    if (this.state.pending || this.state.busy || !this.state.draft) return;
    const request = this.draftScope(); this.publish({ busy: true, error: '' });
    try { const validation = await this.port.validateDraft(request); if (!validation.valid) throw new Error(validation.diagnostics.map(d => `${d.field}: ${d.code} (${d.repair})`).join('; ')); const preview = await this.port.previewDraft(request); this.publish({ preview, message: 'Preview only. Answers here do not record learning progress.' }); }
    catch (error) { this.publish({ error: message(error) }); } finally { this.publish({ busy: false }); }
  }
  private accept(receipt: LessonReceipt) { this.undoStack = []; this.redoStack = []; this.publish({ saved: receipt.saved, composition: receipt.saved.composition, draft: null, preview: null, drafts: this.state.drafts.filter(d => d.id !== this.state.draft?.id), message: 'Lesson saved locally. Existing study snapshots keep their original revisions.' }); }
  async save() {
    if (this.state.pending || this.state.busy || !this.state.draft) return;
    await this.preview(); const { preview, draft } = this.state; if (!preview || !draft || preview.draftRevision !== draft.revision) return;
    const request: ApplyLessonDraft = { ...this.draftScope(), expectedHeadRevision: draft.baseHeadRevision, previewHash: preview.hash, operationId: crypto.randomUUID() };
    let attempted = false;
    await this.execute(async () => {
      if (attempted) { const receipt = await this.port.getReceipt(request.operationId); if (receipt) { this.accept(receipt); return; } }
      attempted = true; this.accept(await this.port.applyDraft(request));
    });
  }
  /** Explicit refresh preserves the old durable draft; creating a new draft is a separate action. */
  async refreshHead() {
    if (this.state.busy) return;
    this.publish({ busy: true, error: '' });
    try { const saved = await this.port.read(this.unitId); this.publish({ saved, message: 'Latest saved lesson checked. Your editing draft is preserved; it has not been rebased or overwritten.' }); }
    catch (error) { this.publish({ error: message(error) }); } finally { this.publish({ busy: false }); }
  }
}

/** Restore through semantic operations, retaining stable node IDs and revision history. */
export function restoreOperations(current: Composition, target: Composition): LessonOperation[] {
  const operations: LessonOperation[] = []; const wanted = new Map(target.nodes.map(n => [n.nodeId, n])); const existing = new Set(current.nodes.map(n => n.nodeId));
  const parents = new Map<string, string>(); current.nodes.forEach(n => Object.values(n.slots).flat().forEach(id => parents.set(id, n.nodeId)));
  current.nodes.filter(n => !wanted.has(n.nodeId) && wanted.has(parents.get(n.nodeId) ?? '')).forEach(n => operations.push({ kind: 'removeNode', id: n.nodeId }));
  const visit = (id: string) => {
    const node = wanted.get(id); if (!node) return;
    if (existing.has(id)) { operations.push({ kind: 'configureNode', id, props: node.props }, { kind: 'configureInteraction', id, on: node.on, ...(node.visible ? { visible: node.visible } : {}) }); }
    Object.entries(node.slots).forEach(([slot, ids]) => ids.forEach((child, index) => {
      const childNode = wanted.get(child); if (!childNode) return;
      if (!existing.has(child)) operations.push({ kind: 'insertNode', parent: id, slot, index, node: { ...childNode, slots: Object.fromEntries(Object.keys(childNode.slots).map(s => [s, []])) } });
      else operations.push({ kind: 'moveNode', id: child, parent: id, slot, index });
      visit(child);
    }));
  };
  target.bindings.forEach(binding => operations.push({ kind: 'bindReference', binding })); visit(target.rootNodeId); operations.push({ kind: 'setAppearance', tokens: target.appearance }); return operations;
}
