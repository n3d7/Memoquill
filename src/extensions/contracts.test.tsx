import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { admitExtension, restoreExtension, type ExtensionScope } from './contracts';
import { ExtensionPlaceholder } from './ExtensionPlaceholder';

const scope: ExtensionScope = { courseId: 'course-a', workspaceId: 'workspace-a', lessonId: 'lesson-a', nodeId: 'node-a' };
const fixture = () => ({ contractVersion: 1, scope, manifest: {
  extensionId: 'example.physics', version: '1.0.0', stateSchemaVersion: 1,
  capabilities: ['course.state.read'], provenance: { kind: 'agent', source: 'Explicit proposal' },
  compatibleContractVersions: [1], migrations: [],
}, state: { schemaVersion: 1, payload: { angle: 0, note: '<script>throw 1</script>' } } });

describe('inert course extension contracts', () => {
  it('preserves exact original bytes and state without granting execution', () => {
    const raw = JSON.stringify(fixture(), null, 2);
    const admitted = admitExtension(raw, scope);
    expect(admitted.status).toBe('inert');
    expect(admitted.activationAllowed).toBe(false);
    expect(restoreExtension(admitted)).toBe(raw);
    expect(admitted.diagnostics).toContain('runtime_unavailable');
  });
  it.each(['courseId', 'workspaceId', 'lessonId', 'nodeId'] as const)('denies a foreign %s before exposing any payload', key => {
    const value = fixture(); value.scope = { ...scope, [key]: 'foreign' };
    const result = admitExtension(JSON.stringify(value), scope);
    expect(result.status).toBe('rejected');
    expect(result.originalJson).toBeUndefined();
    expect(result.diagnostics).toContain('scope_denied');
  });
  it('retains unknown versions, capabilities and state versions inertly', () => {
    const value = { ...fixture(), contractVersion: 7, extra: { untouched: true } };
    value.manifest.capabilities = ['native.filesystem']; value.state.schemaVersion = 9;
    const raw = JSON.stringify(value);
    const result = admitExtension(raw, scope);
    expect(restoreExtension(result)).toBe(raw);
    expect(result.activationAllowed).toBe(false);
    expect(result.diagnostics).toContain('unsupported_contract');
  });
  it('does not activate capabilities or migrations declared by generated content', () => {
    const value = fixture(); value.manifest.capabilities = ['native.filesystem'];
    const result = admitExtension(JSON.stringify(value), scope);
    expect(result.diagnostics).toContain('capability_unavailable');
    expect(result.activationAllowed).toBe(false);
    expect(result.status).toBe('inert');
  });
  it.each([{ toString: null }, ['agent']])('preserves malformed provenance inertly without coercing supplied objects', kind => {
    const value = fixture();
    const raw = JSON.stringify({ ...value, manifest: { ...value.manifest, provenance: { kind, source: 'Imported' } } });
    const result = admitExtension(raw, scope);
    expect(result.status).toBe('inert');
    expect(result.diagnostics).toContain('invalid_manifest');
    expect(restoreExtension(result)).toBe(raw);
  });
  it('rejects malformed, oversized and deeply nested input without retaining unsafe data', () => {
    for (const raw of ['{', '"' + 'a'.repeat(65536) + '"', '['.repeat(20) + '0' + ']'.repeat(20)]) {
      const result = admitExtension(raw, scope);
      expect(result.status).toBe('rejected'); expect(result.originalJson).toBeUndefined();
    }
  });
  it('renders passive readable recovery information with no executable markup', () => {
    const raw = JSON.stringify(fixture());
    render(<ExtensionPlaceholder admission={admitExtension(raw, scope)} />);
    expect(screen.getByRole('note')).toHaveTextContent('preserved');
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
    expect(document.querySelector('script,iframe,object')).toBeNull();
  });
});
