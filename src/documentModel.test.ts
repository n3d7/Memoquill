import { describe, expect, it } from 'vitest';
import { roundTrip, type Document } from './documentModel';

describe('canonical document editor contract', () => {
  it('preserves block identity, marks and exact multilingual/code/math/table/media content', () => {
    const p = (id: string, text: string) => ({ id, content: { type: 'paragraph' as const, inlines: [{ text, marks: [] }] } });
    const document: Document = { schemaVersion: 1, blocks: [
      { id: 'heading', content: { type: 'heading', level: 2, inlines: [{ text: 'café 世界 e\u0301 😀 אב', marks: [{ type: 'strong' }] }] } },
      { id: 'code', content: { type: 'code', text: 'def f():\n    return "😀"\n', language: 'python' } },
      { id: 'math', content: { type: 'math', source: 'x^2+\\alpha' } },
      { id: 'list', content: { type: 'list', ordered: true, items: [[p('item', 'remember')]] } },
      { id: 'table', content: { type: 'table', rows: [[[p('cell', 'value')]]] } },
      { id: 'media', content: { type: 'media', assetDigest: 'a'.repeat(64), mime: 'image/png', caption: 'caption 世界', alt: 'diagram' } },
      { id: 'unknown', content: { type: 'unsupported', original: '<script>inert</script>', source: { assetDigest: 'b'.repeat(64), locator: 'original' }, reason: 'Unsupported format' } },
    ] };
    expect(roundTrip(document)).toEqual(document);
  });
});
