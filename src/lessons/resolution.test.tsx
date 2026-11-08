import { act, render, screen, waitFor } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { useResolvedLesson } from './LessonComposer';
import { composition } from './testFixtures';
import type { Composition, LessonPort } from './types';
import type { SavedDocument } from '../application/contentTypes';
it('reads durable Unit-scoped resource pins and ignores an old asynchronous result after scope changes', async () => {
  let finish: (resource: SavedDocument) => void = () => {};
  const readResource = vi.fn((unitId: string) => unitId === 'english' ? new Promise<SavedDocument>(resolve => { finish = resolve; }) : Promise.resolve({ id: 'physics-pin', resourceId: 'physics-resource', revision: 1, document: { schemaVersion: 1 as const, blocks: [{ id: 'block', content: { type: 'paragraph' as const, inlines: [{ text: 'Physics result', marks: [] }] } }] } }));
  const port = { readResource } as unknown as LessonPort;
  const make = (unitId: string) => { const c = composition(unitId); c.bindings = [{ bindingId: 'same-binding', targetKind: 'resource', targetId: 'same-resource', revisionId: `${unitId}-pin` }]; return c; };
  function Harness({ c }: { c: Composition }) { const { resolved } = useResolvedLesson(port, c, 'reopened-draft'); return <output>{Object.values(resolved.resources).map(resource => resource?.id).join(',')}</output>; }
  const { rerender } = render(<Harness c={make('english')} />); rerender(<Harness c={make('physics')} />); await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('physics-pin'));
  await act(async () => finish({ id: 'english-pin', resourceId: 'same-resource', revision: 1, document: { schemaVersion: 1, blocks: [] } }));
  expect(screen.getByRole('status')).toHaveTextContent('physics-pin'); expect(readResource.mock.calls).toEqual([['english', 'english-pin'], ['physics', 'physics-pin']]);
});
