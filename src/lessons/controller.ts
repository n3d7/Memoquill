import type { ApplyLessonDraft, Composition, LessonDraft, LessonOperation, LessonPort, LessonPreview, LessonReceipt, RegistryDescriptor, SavedComposition } from './types';

export interface LessonState {
  composition: Composition | null; draft: LessonDraft | null; drafts: LessonDraft[];
  saved: SavedComposition | null; preview: LessonPreview | null; registry: RegistryDescriptor[];
  busy: boolean; pending: boolean; conflict: boolean; message: string; error: string; canUndo: boolean; canRedo: boolean;
}
interface History { before: Composition; after: Composition }
const message = (error: unknown) => error instanceof Error ? error.message : typeof error === 'object' && error && 'code' in error ? String(error.code) : 'The lesson change could not be confirmed.';
/** One durable authoring scope per Unit. Unknown outcomes retain the exact request. */
export class LessonController {
  private state: LessonState = { composition: null, draft: null, drafts: [], saved: null, preview: null, registry: [], busy: false, pending: false, conflict: false, message: '', error: '', canUndo: false, canRedo: false };
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
    try { await work(); this.pending = null; this.publish({ pending: false, conflict: false }); }
    catch (error) {
      const code = typeof error === 'object' && error && 'code' in error ? String(error.code).toLowerCase() : '';
      const conflict = /revision[_ ]?conflict/i.test(code || message(error));
      const rejected = ['invalid_request', 'invalid_reference', 'invalid_tree', 'unsupported_version', 'permission_denied', 'budget_exceeded', 'needs_review', 'capability_unavailable'].includes(code);
      if (rejected) this.pending = null;
      this.publish({ pending: !rejected, conflict, error: `${message(error)} Your draft is preserved. ${rejected ? 'Correct the form and try again.' : 'Retry the same change or check for a conflict.'}`, message: '' });
    }
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
      if (draft.id !== request.draftId || draft.unitId !== request.unitId || draft.revision !== request.expectedDraftRevision + 1) throw new Error('The editing reply does not match the pending lesson change.');
      if (history === 'normal' && JSON.stringify(before) !== JSON.stringify(draft.composition)) { this.undoStack.push({ before, after: structuredClone(draft.composition) }); this.redoStack = []; }
      if (history === 'undo') { const entry = this.undoStack.pop(); if (entry) this.redoStack.push(entry); }
      if (history === 'redo') { const entry = this.redoStack.pop(); if (entry) this.undoStack.push(entry); }
      this.publish({ draft, drafts: this.state.drafts.map(d => d.id === draft.id ? draft : d), composition: draft.composition, preview: null, message: 'Editing draft saved locally.' });
    });
  }
  async undo() { const entry = this.undoStack.at(-1); if (entry && !this.state.draft) await this.begin(); if (entry && this.state.composition) await this.edit(restoreOperations(this.state.composition, entry.before), 'undo'); }
  async redo() { const entry = this.redoStack.at(-1); if (entry && !this.state.draft) await this.begin(); if (entry && this.state.composition) await this.edit(restoreOperations(this.state.composition, entry.after), 'redo'); }
  async preview() {
    if (this.state.pending || this.state.busy || !this.state.draft) return;
    const request = this.draftScope(); this.publish({ busy: true, error: '', preview: null });
    try { const validation = await this.port.validateDraft(request); if (!validation.valid) throw new Error(validation.diagnostics.map(d => `${d.field}: ${d.code} (${d.repair})`).join('; ')); const preview = await this.port.previewDraft(request); if (preview.draftId !== request.draftId || preview.draftRevision !== request.expectedDraftRevision || preview.composition.unitId !== this.unitId) throw new Error('The preview does not match this editing draft.'); this.publish({ preview, message: 'Preview only. Answers here do not record learning progress.' }); }
    catch (error) { const code = typeof error === 'object' && error && 'code' in error ? String(error.code) : message(error); this.publish({ error: message(error), conflict: /revision[_ ]?conflict/i.test(code) }); } finally { this.publish({ busy: false }); }
  }
  private accept(receipt: LessonReceipt) { this.publish({ saved: receipt.saved, composition: receipt.saved.composition, draft: null, preview: null, drafts: this.state.drafts.filter(d => d.id !== this.state.draft?.id), message: 'Lesson saved locally. Existing study snapshots keep their original revisions.' }); }
  async save() {
    if (this.state.pending || this.state.busy || !this.state.draft) return;
    await this.preview(); const { preview, draft } = this.state; if (!preview || !draft || preview.draftRevision !== draft.revision) return;
    const request: ApplyLessonDraft = { ...this.draftScope(), expectedHeadRevision: draft.baseHeadRevision, previewHash: preview.hash, operationId: crypto.randomUUID() };
    let attempted = false;
    const accept = (receipt: LessonReceipt) => {
      if (!receipt.committed || receipt.operationId !== request.operationId || receipt.proposalHash !== request.previewHash || receipt.saved.unitId !== this.unitId || receipt.saved.composition.unitId !== this.unitId) throw new Error('The save receipt does not match the pending lesson change.');
      this.accept(receipt);
    };
    await this.execute(async () => {
      if (attempted) { const receipt = await this.port.getReceipt(request.operationId); if (receipt) { accept(receipt); return; } }
      attempted = true; accept(await this.port.applyDraft(request));
    });
  }
  /** Explicit refresh preserves the old durable draft; creating a new draft is a separate action. */
  async refreshHead() {
    if (this.state.busy) return;
    this.publish({ busy: true, error: '' });
    try { const saved = await this.port.read(this.unitId); this.publish({ saved, message: 'Latest saved lesson checked. Your editing draft is preserved; it has not been rebased or overwritten.' }); }
    catch (error) { this.publish({ error: message(error) }); } finally { this.publish({ busy: false }); }
  }
  async startLatestDraft() {
    if (this.state.busy || this.state.pending && !this.state.conflict) return;
    this.publish({ busy: true, error: '' });
    let saved: SavedComposition | null;
    try { saved = await this.port.read(this.unitId); } catch (error) { this.publish({ busy: false, error: message(error) }); return; }
    // Explicitly abandon only a confirmed conflict; the original draft remains durable.
    this.pending = null; this.publish({ pending: false, conflict: false, saved, busy: false });
    const request = { ...this.scope(), operationId: crypto.randomUUID(), expectedHeadRevision: saved?.id ?? null };
    await this.execute(async () => { const draft = await this.port.createDraft(request); this.undoStack = []; this.redoStack = []; this.publish({ draft, composition: draft.composition, drafts: [...this.state.drafts.filter(d => d.id !== this.state.draft?.id), ...(this.state.draft ? [this.state.draft] : []), draft], preview: null, message: 'Started a new draft from the latest saved lesson. The earlier draft remains available for comparison; no edits were silently rebased.' }); });
  }
}

/** Restore through semantic operations, retaining stable node IDs and revision history. */
export function restoreOperations(current: Composition, target: Composition): LessonOperation[] {
  const operations: LessonOperation[] = []; const wanted = new Map(target.nodes.map(n => [n.nodeId, n])); const existing = new Set(current.nodes.map(n => n.nodeId));
  const parents = new Map<string, string>(); current.nodes.forEach(n => Object.values(n.slots).flat().forEach(id => parents.set(id, n.nodeId)));
  current.nodes.filter(n => !wanted.has(n.nodeId) && wanted.has(parents.get(n.nodeId) ?? '')).forEach(n => operations.push({ kind: 'removeNode', id: n.nodeId }));
  const positions = new Map<string, string>(); current.nodes.forEach(n => Object.entries(n.slots).forEach(([slot, ids]) => ids.forEach((id, index) => positions.set(id, JSON.stringify([n.nodeId, slot, index])))));
  const collect = (id: string): typeof target.nodes => { const node = wanted.get(id); return node ? [node, ...Object.values(node.slots).flat().flatMap(collect)] : []; };
  const visit = (id: string) => {
    const node = wanted.get(id); if (!node) return;
    const old = current.nodes.find(n => n.nodeId === id);
    if (old && JSON.stringify(old.props) !== JSON.stringify(node.props)) operations.push({ kind: 'configureNode', id, props: node.props });
    if (old && JSON.stringify([old.on, old.visible]) !== JSON.stringify([node.on, node.visible])) operations.push({ kind: 'configureInteraction', id, on: node.on, ...(node.visible ? { visible: node.visible } : {}) });
    Object.entries(node.slots).forEach(([slot, ids]) => ids.forEach((child, index) => {
      const childNode = wanted.get(child); if (!childNode) return;
      if (!existing.has(child)) {
        const subtree = collect(child);
        if (subtree.every(n => !existing.has(n.nodeId))) { operations.push({ kind: 'insertSubtree', parent: id, slot, index, rootNodeId: child, nodes: subtree }); return; }
        operations.push({ kind: 'insertNode', parent: id, slot, index, node: { ...childNode, slots: Object.fromEntries(Object.keys(childNode.slots).map(s => [s, []])) } });
      } else if (positions.get(child) !== JSON.stringify([id, slot, index])) operations.push({ kind: 'moveNode', id: child, parent: id, slot, index });
      visit(child);
    }));
  };
  target.bindings.filter(binding => JSON.stringify(current.bindings.find(b => b.bindingId === binding.bindingId)) !== JSON.stringify(binding)).forEach(binding => operations.push({ kind: 'bindReference', binding })); visit(target.rootNodeId); if (JSON.stringify(current.appearance) !== JSON.stringify(target.appearance)) operations.push({ kind: 'setAppearance', tokens: target.appearance }); return operations;
}
