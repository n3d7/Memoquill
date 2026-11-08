import { Schema, type Node as EditorNode, type Mark as EditorMark } from 'prosemirror-model';

export interface Inline { text: string; marks: Mark[] }
export type Mark = { type: 'strong' | 'emphasis' | 'code' } | { type: 'link'; href: string };
export interface Block { id: string; content: BlockContent }
export type BlockContent =
  | { type: 'paragraph'; inlines: Inline[] }
  | { type: 'heading'; level: number; inlines: Inline[] }
  | { type: 'list'; ordered: boolean; items: Block[][] }
  | { type: 'code'; text: string; language?: string | null }
  | { type: 'math'; source: string }
  | { type: 'table'; rows: Block[][][] }
  | { type: 'media'; assetDigest: string; mime: string; caption: string; alt: string }
  | { type: 'unsupported'; original: string; source?: { assetDigest: string; locator: string } | null; reason: string };
export interface Document { schemaVersion: 1; blocks: Block[] }

export function safeLink(value: string): boolean {
  try { const url = new URL(value); return ['https:', 'http:', 'mailto:'].includes(url.protocol) && !/[\u0000-\u0020\u007f]/.test(value); }
  catch { return false; }
}

export const editorSchema = new Schema({
  nodes: {
    doc: { content: 'block+' },
    paragraph: { group: 'block', content: 'text*', attrs: { id: { default: null } }, toDOM: () => ['p', 0] },
    heading: { group: 'block', content: 'text*', attrs: { id: { default: null }, level: { default: 2 } }, toDOM: node => [`h${node.attrs.level}`, 0] },
    code_block: { group: 'block', content: 'text*', marks: '', code: true, attrs: { id: { default: null }, language: { default: null } }, toDOM: () => ['pre', ['code', 0]] },
    preserved: { group: 'block', atom: true, attrs: { id: { default: null }, content: {} }, toDOM: node => ['div', { class: 'preserved-block' }, blockLabel(node.attrs.content as BlockContent)] },
    text: {},
  },
  marks: {
    strong: { toDOM: () => ['strong', 0] },
    emphasis: { toDOM: () => ['em', 0] },
    code: { toDOM: () => ['code', 0] },
    link: { attrs: { href: {} }, toDOM: () => ['span', { class: 'document-link' }, 0] },
  },
});

function blockLabel(content: BlockContent): string {
  if (content.type === 'math') return `Formula: ${content.source}`;
  if (content.type === 'media') return `Attachment: ${content.caption || content.alt}`;
  if (content.type === 'unsupported') return `Preserved source: ${content.reason}`;
  return `${content.type}: edit with the structured controls below`;
}
function textNodes(inlines: Inline[]): EditorNode[] {
  return inlines.filter(inline => inline.text).map(inline => editorSchema.text(inline.text, inline.marks.map(mark => {
    if (mark.type === 'link' && !safeLink(mark.href)) throw new Error('Unsupported link URL');
    return editorSchema.marks[mark.type].create(mark.type === 'link' ? { href: mark.href } : undefined);
  })));
}
function editorBlock(block: Block): EditorNode {
  const content = block.content;
  if (content.type === 'paragraph' || content.type === 'heading') {
    return editorSchema.node(content.type, { id: block.id, ...('level' in content ? { level: content.level } : {}) }, textNodes(content.inlines));
  }
  if (content.type === 'code') return editorSchema.node('code_block', { id: block.id, language: content.language ?? null }, content.text ? editorSchema.text(content.text) : undefined);
  return editorSchema.node('preserved', { id: block.id, content: structuredClone(content) });
}
export function toEditorDoc(document: Document): EditorNode {
  if (document.schemaVersion !== 1 || !document.blocks.length) throw new Error('Unsupported or empty document');
  const result = editorSchema.node('doc', null, document.blocks.map(editorBlock));
  result.check();
  return result;
}
function canonicalMark(mark: EditorMark): Mark {
  return mark.type.name === 'link' ? { type: 'link', href: String(mark.attrs.href) } : { type: mark.type.name as 'strong' | 'emphasis' | 'code' };
}
function canonicalBlock(node: EditorNode): Block {
  const id = String(node.attrs.id ?? crypto.randomUUID());
  if (node.type.name === 'preserved') return { id, content: structuredClone(node.attrs.content as BlockContent) };
  if (node.type.name === 'code_block') return { id, content: { type: 'code', text: node.textContent, language: node.attrs.language as string | null } };
  const inlines: Inline[] = [];
  node.forEach(text => inlines.push({ text: text.text ?? '', marks: text.marks.map(canonicalMark) }));
  return { id, content: node.type.name === 'heading' ? { type: 'heading', level: Number(node.attrs.level), inlines } : { type: 'paragraph', inlines } };
}
export function fromEditorDoc(document: EditorNode): Document {
  const blocks: Block[] = []; document.forEach(node => blocks.push(canonicalBlock(node)));
  return { schemaVersion: 1, blocks };
}
export function roundTrip(document: Document): Document { return fromEditorDoc(toEditorDoc(document)); }
export function emptyDocument(): Document {
  return { schemaVersion: 1, blocks: [{ id: crypto.randomUUID(), content: { type: 'paragraph', inlines: [] } }] };
}
