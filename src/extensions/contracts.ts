/** Data contracts only. A manifest never supplies executable handlers or authority. */
export interface ExtensionScope { courseId: string; workspaceId: string; lessonId: string; nodeId: string }
export type ExtensionCapability = 'course.state.read' | 'course.state.write' | 'lesson.content.read' | 'lesson.response.write';
export interface ExtensionManifest {
  extensionId: string;
  version: string;
  stateSchemaVersion: number;
  capabilities: ExtensionCapability[];
  provenance: { kind: 'manual' | 'agent' | 'import'; source: string };
  compatibleContractVersions: number[];
  /** Identifiers of reviewed host migrations, never code supplied by the extension. */
  migrations: { id: string; from: number; to: number }[];
}
export interface ExtensionEnvelope {
  contractVersion: 1;
  scope: ExtensionScope;
  manifest: ExtensionManifest;
  state: { schemaVersion: number; payload: unknown };
}
export interface ExtensionAdmission {
  status: 'inert' | 'rejected';
  activationAllowed: false;
  diagnostics: string[];
  /** Exact bounded source. Retained only after owner-scope validation. */
  originalJson?: string;
}
const capabilities = new Set<string>(['course.state.read', 'course.state.write', 'lesson.content.read', 'lesson.response.write']);
const object = (value: unknown): value is Record<string, unknown> => value !== null && typeof value === 'object' && !Array.isArray(value);
const positive = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
const text = (value: unknown, maximum = 256): value is string => typeof value === 'string' && value.trim().length > 0 && value.length <= maximum;
const keys = (value: Record<string, unknown>, expected: string[]) => Object.keys(value).length === expected.length && expected.every(key => Object.hasOwn(value, key));

function bounded(value: unknown, depth = 0, budget = { remaining: 4096 }): boolean {
  if (depth > 16 || --budget.remaining < 0) return false;
  if (typeof value === 'number') return Number.isFinite(value);
  if (Array.isArray(value)) return value.every(child => bounded(child, depth + 1, budget));
  return !object(value) || Object.values(value).every(child => bounded(child, depth + 1, budget));
}
function validManifest(value: unknown): value is ExtensionManifest {
  if (!object(value) || !keys(value, ['extensionId', 'version', 'stateSchemaVersion', 'capabilities', 'provenance', 'compatibleContractVersions', 'migrations'])) return false;
  const p = value.provenance;
  return text(value.extensionId) && text(value.version, 64) && positive(value.stateSchemaVersion)
    && Array.isArray(value.capabilities) && value.capabilities.length <= 16 && value.capabilities.every(item => text(item, 128))
    && new Set(value.capabilities).size === value.capabilities.length
    && object(p) && keys(p, ['kind', 'source']) && typeof p.kind === 'string' && ['manual', 'agent', 'import'].includes(p.kind) && text(p.source, 4096)
    && Array.isArray(value.compatibleContractVersions) && value.compatibleContractVersions.length <= 16 && value.compatibleContractVersions.every(positive)
    && Array.isArray(value.migrations) && value.migrations.length <= 16 && value.migrations.every(m => object(m) && keys(m, ['id', 'from', 'to']) && text(m.id) && positive(m.from) && positive(m.to) && m.to > m.from);
}

/** Caller supplies trusted scope from its bound service, never from the manifest. */
export function admitExtension(originalJson: string, expectedScope: ExtensionScope): ExtensionAdmission {
  const reject = (reason: string): ExtensionAdmission => ({ status: 'rejected', activationAllowed: false, diagnostics: [reason] });
  if (new TextEncoder().encode(originalJson).byteLength > 65536) return reject('budget_exceeded');
  let raw: unknown;
  try { raw = JSON.parse(originalJson); } catch { return reject('invalid_json'); }
  if (!bounded(raw)) return reject('budget_exceeded');
  if (!object(raw) || !object(raw.scope)) return reject('invalid_envelope');
  for (const key of ['courseId', 'workspaceId', 'lessonId', 'nodeId'] as const) {
    if (!text(expectedScope[key], 128) || raw.scope[key] !== expectedScope[key]) return reject('scope_denied');
  }
  const diagnostics = ['runtime_unavailable'];
  if (raw.contractVersion !== 1) diagnostics.push('unsupported_contract');
  else {
    if (!keys(raw, ['contractVersion', 'scope', 'manifest', 'state']) || !keys(raw.scope, ['courseId', 'workspaceId', 'lessonId', 'nodeId']) || !validManifest(raw.manifest)) diagnostics.push('invalid_manifest');
    else {
      if (!raw.manifest.capabilities.every(capability => capabilities.has(capability))) diagnostics.push('capability_unavailable');
      if (!raw.manifest.compatibleContractVersions.includes(1)) diagnostics.push('incompatible_manifest');
      if (!object(raw.state) || !keys(raw.state, ['schemaVersion', 'payload']) || raw.state.schemaVersion !== raw.manifest.stateSchemaVersion) diagnostics.push('state_migration_required');
      if (raw.manifest.migrations.length > 0) diagnostics.push('migration_unavailable');
    }
  }
  return { status: 'inert', activationAllowed: false, diagnostics, originalJson };
}

/** Recovery/export projection, not activation or a write of learner history. */
export function restoreExtension(admission: ExtensionAdmission): string | undefined { return admission.originalJson; }
