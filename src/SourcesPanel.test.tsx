import { render } from '../tests/courseRender';
import { beforeEach, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SourcesPanel } from './SourcesPanel';
import type { SaveSource, SelectedSource } from './sourceBridge';

const native = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/core', () => native);
const selected = { filename: 'notes.txt', digest: 'a'.repeat(64), byteLength: 12 };
const original = { ...selected, id: 'source', resourceId: 'resource', revision: 1, mediaType: 'application/octet-stream', attribution: null, license: null, originUrl: null };
beforeEach(() => {
  native.invoke.mockReset();
  native.invoke.mockImplementation(async (command: string) => {
    if (command === 'sources') return { revision: 0, sources: [] };
    if (command === 'pick_source') return selected;
    throw new Error('Unexpected native command');
  });
});

it('shows extracted source records and honest partial coverage without executing markup', async () => {
  const markup = '<img src=x onerror=alert(1)>';
  native.invoke.mockImplementation(async (command: string) => command === 'sources'
    ? { revision: 1, sources: [original] }
    : { source: original, text: 'raw csv', hex: null, truncated: false, extraction: {
      contractVersion: 1, format: 'csv', status: 'partial', diagnostic: 'recordLimit',
      records: [{ locator: { kind: 'csvRecord', value: '2' }, fields: [markup, '=1+2'] }],
    } });
  const { container } = render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  await userEvent.setup().click(await screen.findByRole('button', { name: /Preview original/ }));
  expect(await screen.findByRole('region', { name: 'Extracted source preview' })).toBeVisible();
  expect(screen.getByText(/Extraction is partial/)).toBeVisible();
  expect(screen.getByText(/record limit/)).toBeVisible();
  expect(screen.getByText('CSV record 2')).toBeVisible();
  expect(screen.getByText(markup)).toBeVisible();
  expect(screen.getByText('=1+2')).toBeVisible();
  expect(container.querySelector('img, script, iframe, a')).toBeNull();
});

it.each([
  ['complete', null, 'Extraction complete'],
  ['failed', 'malformedInput', 'Extraction failed'],
  ['attachmentOnly', 'unsupportedFormat', 'Original only'],
])('shows %s extraction status while retaining the raw original preview', async (status, diagnostic, message) => {
  native.invoke.mockImplementation(async (command: string) => command === 'sources'
    ? { revision: 1, sources: [original] }
    : { source: original, text: 'raw original', hex: null, truncated: false,
      extraction: { contractVersion: 1, format: 'json', status, diagnostic, records: [] } });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  await userEvent.setup().click(await screen.findByRole('button', { name: /Preview original/ }));
  expect(await screen.findByText(new RegExp(message))).toBeVisible();
  expect(screen.getByText('raw original')).toBeVisible();
});

it('retains raw preview when an extraction contract is newer than supported', async () => {
  native.invoke.mockImplementation(async (command: string) => command === 'sources'
    ? { revision: 1, sources: [original] }
    : { source: original, text: 'raw original', hex: null, truncated: false,
      extraction: { contractVersion: 2, format: 'json', status: 'complete', diagnostic: null,
        records: [{ locator: { kind: 'jsonPointer', value: '/0' }, fields: ['uninterpreted future record'] }] } });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  await userEvent.setup().click(await screen.findByRole('button', { name: /Preview original/ }));
  expect(await screen.findByText(/Extraction preview is unavailable in this version/)).toBeVisible();
  expect(screen.getByText('raw original')).toBeVisible();
  expect(screen.queryByText('uninterpreted future record')).not.toBeInTheDocument();
});

it('cancels file selection without saving and restores the chooser focus', async () => {
  native.invoke.mockImplementation(async (command: string) => command === 'sources' ? { revision: 0, sources: [] } : null);
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  const user = userEvent.setup();
  const chooser = await screen.findByRole('button', { name: 'Choose source file' });
  await user.click(chooser);
  await waitFor(() => expect(chooser).toHaveFocus());
  expect(native.invoke.mock.calls.map(call => call[0])).toEqual(['sources', 'pick_source']);
  expect(screen.queryByRole('button', { name: 'Attach selected source' })).not.toBeInTheDocument();
});

it('retains the exact failed request, freezes its metadata, and retries the durable operation', async () => {
  let attempts = 0;
  native.invoke.mockImplementation(async (command: string, args?: { request: SaveSource }) => {
    if (command === 'sources') return { revision: 4, sources: [] };
    if (command === 'pick_source') return selected;
    if (command === 'save_source') {
      attempts++;
      if (attempts === 1) throw { code: 'outcome_unconfirmed' };
      return { ...original, ...args!.request.source, revision: 5 };
    }
    throw new Error('Unexpected native command');
  });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Choose source file' }));
  await user.type(screen.getByLabelText('Attribution'), 'Author');
  await user.type(screen.getByLabelText('License'), 'CC BY');
  await user.type(screen.getByLabelText('Source URL'), 'https://example.invalid/source');
  await user.click(screen.getByRole('button', { name: 'Attach selected source' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('unconfirmed');
  expect(screen.getByLabelText('Attribution')).toBeDisabled();
  await user.click(screen.getByRole('button', { name: 'Retry same attachment' }));
  const saves = native.invoke.mock.calls.filter(call => call[0] === 'save_source');
  expect(saves[1][1]).toEqual(saves[0][1]);
  expect(saves[0][1].request).toMatchObject({ contractVersion: 1, workspaceId: 'workspace', expectedRevision: 4, source: { resourceId: 'resource', digest: selected.digest, filename: 'notes.txt', mediaType: 'application/octet-stream', attribution: 'Author', license: 'CC BY', originUrl: 'https://example.invalid/source' } });
  expect(saves[0][1].request.operationId).toBeTruthy();
  expect(saves[0][1].request.source.id).toBeTruthy();
  expect(await screen.findByText('Attached original source.')).toBeVisible();
});

it('allows an explicit discard of a failed attachment before another choice', async () => {
  const onPendingChange = vi.fn();
  native.invoke.mockImplementation(async (command: string) => {
    if (command === 'sources') return { revision: 0, sources: [] };
    if (command === 'pick_source') return selected;
    throw { code: 'invalid_request' };
  });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" onPendingChange={onPendingChange} />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Choose source file' }));
  await user.click(screen.getByRole('button', { name: 'Attach selected source' }));
  await screen.findByRole('alert');
  expect(onPendingChange).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole('button', { name: 'Discard pending attachment' }));
  expect(onPendingChange).toHaveBeenLastCalledWith(false);
  expect(screen.getByRole('button', { name: 'Choose source file' })).toBeEnabled();
  expect(screen.queryByRole('button', { name: 'Retry same attachment' })).not.toBeInTheDocument();
});

it('shows hostile filenames, provenance and original markup only as passive text', async () => {
  const hostile = { ...original, filename: '<img src=x onerror=alert(1)>.svg', attribution: '<script>alert(2)</script>', originUrl: 'javascript:alert(3)' };
  const markup = '<svg onload="alert(4)"><script>alert(5)</script></svg>';
  native.invoke.mockImplementation(async (command: string) => command === 'sources' ? { revision: 1, sources: [hostile] } : { source: hostile, text: markup, hex: null, truncated: true });
  const { container } = render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: /Preview original/ }));
  expect(await screen.findByText(markup)).toBeVisible();
  expect(screen.getByText(hostile.filename)).toBeVisible();
  expect(screen.getByText(hostile.attribution)).toBeVisible();
  expect(screen.getByText(hostile.originUrl)).toBeVisible();
  expect(screen.getByText(/Preview is truncated/)).toBeVisible();
  expect(container.querySelector('img, script, svg, iframe, embed, object, a')).toBeNull();
});

it('describes unsupported binary previews and retains originals after preview failure', async () => {
  const onPendingChange = vi.fn();
  native.invoke.mockImplementation(async (command: string) => command === 'sources' ? { revision: 1, sources: [original] } : { source: original, text: null, hex: '00 ff 01', truncated: false });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" onPendingChange={onPendingChange} />);
  const user = userEvent.setup();
  const preview = await screen.findByRole('button', { name: /Preview original/ });
  await user.click(preview);
  expect(await screen.findByText('00 ff 01')).toBeVisible();
  expect(screen.getByText(/Binary source/)).toBeVisible();
  native.invoke.mockRejectedValueOnce({ code: 'corrupt_asset' });
  await user.click(preview);
  expect(await screen.findByRole('alert')).toHaveTextContent('Preview unavailable');
  expect(screen.getByText('notes.txt')).toBeVisible();
  expect(preview).toBeEnabled();
  expect(onPendingChange).not.toHaveBeenCalledWith(true);
});

it('blocks duplicate submissions and metadata changes while an attachment is saving', async () => {
  let finish: (value: typeof original) => void = () => { throw new Error('No pending save'); };
  native.invoke.mockImplementation(async (command: string) => {
    if (command === 'sources') return { revision: 0, sources: [] };
    if (command === 'pick_source') return selected;
    if (command === 'save_source') return new Promise<typeof original>(resolve => { finish = resolve; });
    throw new Error('Unexpected native command');
  });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Choose source file' }));
  await user.click(screen.getByRole('button', { name: 'Attach selected source' }));
  expect(screen.getByLabelText('License')).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Discard pending attachment' })).toBeDisabled();
  await user.click(screen.getByRole('button', { name: 'Retry same attachment' }));
  expect(native.invoke.mock.calls.filter(call => call[0] === 'save_source')).toHaveLength(1);
  finish(original);
  expect(await screen.findByText('Attached original source.')).toBeVisible();
});

it('ignores a previous resource chooser result after the resource changes', async () => {
  let finish: (value: SelectedSource) => void = () => { throw new Error('No pending selection'); };
  native.invoke.mockImplementation(async (command: string) => {
    if (command === 'sources') return { revision: 0, sources: [] };
    if (command === 'pick_source') return new Promise<SelectedSource>(resolve => { finish = resolve; });
    throw new Error('Unexpected native command');
  });
  const view = render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Choose source file' }));
  view.rerender(<SourcesPanel workspaceId="workspace" resourceId="other-resource" />);
  finish(selected);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Choose source file' })).toBeEnabled());
  expect(screen.queryByText(/Selected:/)).not.toBeInTheDocument();
  expect(native.invoke.mock.calls.filter(call => call[0] === 'save_source')).toHaveLength(0);
});

it('protects selected and unconfirmed drafts until a save succeeds or the draft is discarded', async () => {
  const onPendingChange = vi.fn();
  let saves = 0;
  native.invoke.mockImplementation(async (command: string) => {
    if (command === 'sources') return { revision: 0, sources: [] };
    if (command === 'pick_source') return selected;
    if (command === 'save_source' && ++saves === 1) throw { code: 'outcome_unconfirmed' };
    if (command === 'save_source') return original;
    throw new Error('Unexpected native command');
  });
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" onPendingChange={onPendingChange} />);
  const user = userEvent.setup();
  const chooser = await screen.findByRole('button', { name: 'Choose source file' });
  expect(onPendingChange).toHaveBeenLastCalledWith(false);
  await user.click(chooser);
  expect(onPendingChange).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole('button', { name: 'Attach selected source' }));
  await screen.findByRole('alert');
  expect(onPendingChange).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole('button', { name: 'Retry same attachment' }));
  await screen.findByText('Attached original source.');
  expect(onPendingChange).toHaveBeenLastCalledWith(false);
  await user.click(chooser);
  expect(onPendingChange).toHaveBeenLastCalledWith(true);
  await user.click(screen.getByRole('button', { name: 'Discard selected source' }));
  expect(onPendingChange).toHaveBeenLastCalledWith(false);
});

it('protects an active picker and releases navigation after cancellation or unmount', async () => {
  const onPendingChange = vi.fn();
  let finish: (value: SelectedSource | null) => void = () => { throw new Error('No pending selection'); };
  native.invoke.mockImplementation(async (command: string) => {
    if (command === 'sources') return { revision: 0, sources: [] };
    if (command === 'pick_source') return new Promise<SelectedSource | null>(resolve => { finish = resolve; });
    throw new Error('Unexpected native command');
  });
  const view = render(<SourcesPanel workspaceId="workspace" resourceId="resource" onPendingChange={onPendingChange} />);
  const user = userEvent.setup();
  const chooser = await screen.findByRole('button', { name: 'Choose source file' });
  await user.click(chooser);
  expect(onPendingChange).toHaveBeenLastCalledWith(true);
  finish(null);
  await waitFor(() => expect(onPendingChange).toHaveBeenLastCalledWith(false));
  await user.click(chooser);
  expect(onPendingChange).toHaveBeenLastCalledWith(true);
  view.unmount();
  expect(onPendingChange).toHaveBeenLastCalledWith(false);
  finish(selected);
});

it('protects unsaved source drafts from window close and releases the close guard on discard', async () => {
  render(<SourcesPanel workspaceId="workspace" resourceId="resource" />);
  const user = userEvent.setup();
  await user.click(await screen.findByRole('button', { name: 'Choose source file' }));
  const pendingClose = new Event('beforeunload', { cancelable: true });
  window.dispatchEvent(pendingClose);
  expect(pendingClose.defaultPrevented).toBe(true);
  await user.click(screen.getByRole('button', { name: 'Discard selected source' }));
  const cleanClose = new Event('beforeunload', { cancelable: true });
  window.dispatchEvent(cleanClose);
  expect(cleanClose.defaultPrevented).toBe(false);
});
