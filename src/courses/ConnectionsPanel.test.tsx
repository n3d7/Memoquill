import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { ConnectionsPanel, type ConnectionsPort } from './ConnectionsPanel';

test('requires a matching pairing challenge and a deliberate per-course permission', async () => {
  const pending = { challenge: 'test-challenge', pid: 42, courseId: null, permissions: null };
  const granted = { ...pending, courseId: 'course-id', permissions: { content: true, progress: false, responses: false, sources: false, originals: false, drafts: false, delegateAdditions: false, delegateDocumentEdits: false } };
  const port: ConnectionsPort = {
    status: vi.fn(async () => ({ enabled: false, peers: [] })),
    setEnabled: vi.fn(async () => ({ enabled: true, peers: [pending] })),
    approve: vi.fn(async () => ({ enabled: true, peers: [granted] })),
    revoke: vi.fn(async () => ({ enabled: true, peers: [] })),
  };
  const user = userEvent.setup();
  render(<ConnectionsPanel port={port} course={{ id: 'course-id', title: 'Synthetic course' }} />);
  await user.click(screen.getByText('Connections'));
  await user.click(await screen.findByRole('button', { name: 'Enable local connections' }));
  const approve = screen.getByRole('button', { name: 'Approve for Synthetic course' });
  await user.type(screen.getByLabelText('Pairing challenge'), pending.challenge);
  expect(approve).toBeDisabled();
  await user.click(screen.getByRole('checkbox', { name: 'Published course structure and normalized document text' }));
  await user.click(approve);
  expect(port.approve).toHaveBeenCalledWith(pending.challenge, 'course-id', { content: true, progress: false, responses: false, sources: false, originals: false, drafts: false, delegateAdditions: false, delegateDocumentEdits: false });
  await user.click(await screen.findByRole('button', { name: 'Revoke bridge 42' }));
  expect(port.revoke).toHaveBeenCalledWith(pending.challenge);
});
