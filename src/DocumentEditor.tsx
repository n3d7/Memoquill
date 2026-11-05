import { createElement, useEffect, useRef, useState, type ReactNode, type RefObject } from 'react';
import { Fragment, Slice } from 'prosemirror-model';
import { EditorState, NodeSelection, Plugin, TextSelection, type Transaction, type Command } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { baseKeymap, toggleMark } from 'prosemirror-commands';
import { history, undo, redo } from 'prosemirror-history';
import { keymap } from 'prosemirror-keymap';
import { editorSchema, emptyDocument, fromEditorDoc, toEditorDoc, type Block, type BlockContent, type Document, type Inline } from './documentModel';
import { MathPreview } from './MathPreview';
import 'prosemirror-view/style/prosemirror.css';

interface Props { document: Document; onChange: (document: Document) => void; readOnly?: boolean; passive?: boolean; label?: string; resume?: { blockId: string; offsetUtf16: number } }
type NewBlockType = 'paragraph' | 'heading' | 'code' | 'math' | 'list' | 'table';
function newBlock(type: NewBlockType): Block {
  const id = crypto.randomUUID();
  switch (type) {
    case 'paragraph': return { id, content: { type, inlines: [] } };
    case 'heading': return { id, content: { type, level: 2, inlines: [] } };
    case 'code': return { id, content: { type, text: '', language: null } };
    case 'math': return { id, content: { type, source: '' } };
    case 'list': return { id, content: { type, ordered: false, items: [[newBlock('paragraph')]] } };
    case 'table': return { id, content: { type, rows: [[[newBlock('paragraph')]]] } };
  }
}

// Split commands copy node attributes. Repair only missing/duplicate identities in the
// same transaction cycle, so identities are durable before the application sees them.
const stableIds = new Plugin({
  appendTransaction(transactions, _old, state) {
    if (!transactions.some(transaction => transaction.docChanged)) return null;
    const seen = new Set<string>();
    const transaction = state.tr;
    state.doc.forEach((node, position) => {
      let id = node.attrs.id as string | null;
      if (!id || seen.has(id)) {
        id = crypto.randomUUID();
        transaction.setNodeMarkup(position, undefined, { ...node.attrs, id });
      }
      seen.add(id);
    });
    return transaction.docChanged ? transaction : null;
  },
});

interface EditorSession { document: Document; readOnly: boolean; onChange: (document: Document) => void }
const attributes = (label: string) => ({ role: 'textbox', 'aria-label': label, 'aria-multiline': 'true', spellcheck: 'true' });
const selectedId = (document: Document, id: string) => document.blocks.some(block => block.id === id) ? id : document.blocks[0].id;

function applyEdit(editor: EditorView, transaction: Transaction, session: EditorSession) {
  if (session.readOnly && transaction.docChanged) return;
  const result = editor.state.applyTransaction(transaction);
  editor.updateState(result.state);
  if (transaction.docChanged) {
    session.document = fromEditorDoc(result.state.doc);
    session.onChange(session.document);
  }
}

function pastedParagraph(line: string) {
  return editorSchema.node('paragraph', { id: crypto.randomUUID() }, line ? editorSchema.text(line) : undefined);
}

function pasteText(editor: EditorView, event: ClipboardEvent, readOnly: boolean) {
  // Intercept before ProseMirror's HTML clipboard parser.
  event.preventDefault();
  if (readOnly) return true;
  const text = event.clipboardData?.getData('text/plain');
  if (text === undefined || text === '') return true;
  const transaction = editor.state.tr;
  if (editor.state.selection.$from.parent.type.spec.code || !/[\r\n]/.test(text)) {
    transaction.insertText(text);
  } else {
    const nodes = text.replace(/\r\n?/g, '\n').split('\n').map(pastedParagraph);
    transaction.replaceSelection(new Slice(Fragment.fromArray(nodes), 1, 1));
  }
  editor.dispatch(transaction.scrollIntoView());
  return true;
}

function preventTransfer(_editor: EditorView, event: Event) { event.preventDefault(); return true; }

function createEditor(host: HTMLDivElement, session: EditorSession, label: string) {
  const editor = new EditorView(host, {
    state: EditorState.create({
      doc: toEditorDoc(session.document),
      plugins: [stableIds, history(), keymap({
        'Mod-b': toggleMark(editorSchema.marks.strong),
        'Mod-i': toggleMark(editorSchema.marks.emphasis),
        'Mod-`': toggleMark(editorSchema.marks.code),
        'Mod-z': undo, 'Mod-Shift-z': redo, 'Mod-y': redo,
      }), keymap(baseKeymap)],
    }),
    editable: () => !session.readOnly,
    attributes: attributes(label),
    dispatchTransaction: transaction => applyEdit(editor, transaction, session),
    handleDOMEvents: {
      paste: (editor, event) => pasteText(editor, event, session.readOnly),
      drop: preventTransfer,
      dragstart: preventTransfer,
    },
  });
  return editor;
}

function syncEditor(editor: EditorView, session: EditorSession, document: Document, label: string, onSync: (document: Document) => void) {
  editor.setProps({ editable: () => !session.readOnly, attributes: attributes(label) });
  if (JSON.stringify(document) === JSON.stringify(session.document)) return;
  const next = toEditorDoc(document);
  const transaction = editor.state.tr.replaceWith(0, editor.state.doc.content.size, next.content).setMeta('addToHistory', false);
  editor.updateState(editor.state.apply(transaction));
  session.document = document;
  onSync(document);
}

function selectBlock(editor: EditorView, blockId: string, offsetUtf16 = 0) {
  let target = -1;
  editor.state.doc.forEach((node, position) => {
    if (node.attrs.id === blockId) target = position;
  });
  if (target < 0) return false;
  const node = editor.state.doc.nodeAt(target)!;
  const offset = Number.isFinite(offsetUtf16) ? Math.max(0, Math.min(node.content.size, Math.trunc(offsetUtf16))) : 0;
  const selection = node.isTextblock
    ? TextSelection.create(editor.state.doc, target + 1 + offset)
    : NodeSelection.create(editor.state.doc, target);
  editor.dispatch(editor.state.tr.setSelection(selection).scrollIntoView());
  return true;
}

function resumeEditor(editor: EditorView, resume: Props['resume'], onSelect: (id: string) => void) {
  if (!resume || !selectBlock(editor, resume.blockId, resume.offsetUtf16)) return;
  onSelect(resume.blockId);
  editor.focus();
}

function replaceDocument(editor: EditorView, session: EditorSession, next: Document) {
  if (session.readOnly) return;
  const replacement = toEditorDoc(next);
  editor.dispatch(editor.state.tr.replaceWith(0, editor.state.doc.content.size, replacement.content));
}

function replaceBlock(editor: EditorView, session: EditorSession, selected: string, content: BlockContent) {
  if (session.readOnly) return;
  editor.state.doc.forEach((node, position) => {
    if (node.attrs.id !== selected) return;
    const replacement = toEditorDoc({ schemaVersion: 1, blocks: [{ id: selected, content }] }).firstChild!;
    editor.dispatch(editor.state.tr.replaceWith(position, position + node.nodeSize, replacement));
  });
}

function runCommand(view: RefObject<EditorView | null>, command: Command) {
  if (!view.current) return;
  command(view.current.state, view.current.dispatch);
  view.current.focus();
}

function useDocumentEditor({ document, onChange, readOnly, label, resume }: Required<Pick<Props, 'document' | 'onChange' | 'readOnly' | 'label'>> & Pick<Props, 'resume'>) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const session = useRef<EditorSession>({ document, readOnly, onChange });
  const [draft, setDraft] = useState(document);
  const [selected, setSelected] = useState(document.blocks[0].id);
  const syncDraft = (next: Document) => {
    setDraft(next);
    setSelected(id => selectedId(next, id));
  };
  session.current.readOnly = readOnly;
  session.current.onChange = next => { syncDraft(next); onChange(next); };

  useEffect(() => {
    const editor = createEditor(host.current!, session.current, label);
    view.current = editor;
    return () => { view.current = null; editor.destroy(); };
    // Keep the view across echoed drafts, callback/status and accessible label changes.
  }, []);
  useEffect(() => {
    syncEditor(view.current!, session.current, document, label, syncDraft);
  }, [document, readOnly, label]);
  useEffect(() => {
    resumeEditor(view.current!, resume, setSelected);
  }, [resume]);
  return { host, view, session, draft, selected, setSelected };
}

function withoutBlock(document: Document, blockId: string): Document {
  const blocks = document.blocks.filter(block => block.id !== blockId);
  return blocks.length ? { schemaVersion: 1, blocks } : emptyDocument();
}

function Formatting({ view, readOnly, label }: { view: RefObject<EditorView | null>; readOnly: boolean; label: string }) {
  return <div role="group" aria-label={`${label} formatting`}>
    {([['strong', 'Bold'], ['emphasis', 'Italic'], ['code', 'Inline code']] as const).map(([mark, text]) =>
      <button key={mark} type="button" disabled={readOnly} onMouseDown={event => event.preventDefault()} onClick={() => runCommand(view, toggleMark(editorSchema.marks[mark]))}>{text}</button>)}
    <button type="button" disabled={readOnly} onClick={() => runCommand(view, undo)}>Undo</button>
    <button type="button" disabled={readOnly} onClick={() => runCommand(view, redo)}>Redo</button>
  </div>;
}

export function DocumentEditor({ document, onChange, readOnly = false, passive = false, label = 'Learning document', resume }: Props) {
  const { host, view, session, draft, selected, setSelected } = useDocumentEditor({ document, onChange, readOnly, label, resume });
  const [adding, setAdding] = useState<NewBlockType>('paragraph');
  const replace = (next: Document) => replaceDocument(view.current!, session.current, next);
  const updateBlock = (content: BlockContent) => replaceBlock(view.current!, session.current, selected, content);
  const block = draft.blocks.find(item => item.id === selected) ?? draft.blocks[0];

  if (passive) return <div className="document-reader" aria-label={label}><CanonicalReading blocks={document.blocks} /></div>;

  return <div className="document-editor">
    {!passive && <><Formatting view={view} readOnly={readOnly} label={label} />
    <p>Use Ctrl/⌘+B for bold, Ctrl/⌘+I for italic, and Ctrl/⌘+Z to undo. Pasted content is plain text. Drops are disabled.</p></>}
    <div className="prose-editor" ref={host} />
    {!passive && <><label>{label === 'Learning document' ? 'Selected block' : `${label} selected block`} <select value={block.id} onChange={event => {
      setSelected(event.target.value);
      selectBlock(view.current!, event.target.value);
    }}>
      {draft.blocks.map((item, index) => <option key={item.id} value={item.id}>{index + 1}. {item.content.type}</option>)}
    </select></label>
    <button type="button" disabled={readOnly} onClick={() => {
      const next = withoutBlock(session.current.document, block.id);
      setSelected(next.blocks[0].id); replace(next);
    }}>Remove selected block</button>
    <BlockFields block={block} onChange={updateBlock} readOnly={readOnly} />
    <label>New block type <select disabled={readOnly} value={adding} onChange={event => setAdding(event.target.value as NewBlockType)}>
      {(['paragraph', 'heading', 'code', 'math', 'list', 'table'] as const).map(type => <option key={type} value={type}>{type}</option>)}
    </select></label>
    <button type="button" disabled={readOnly} onClick={() => {
      const added = newBlock(adding);
      replace({ schemaVersion: 1, blocks: [...session.current.document.blocks, added] });
      setSelected(added.id);
    }}>Add block</button></>}
  </div>;
}

/** Passive mode shares the canonical codec and safe previews with the authoring primitive. */
function CanonicalReading({ blocks }: { blocks: Block[] }) {
  const inlines = (items: Inline[]) => items.map((inline, index) => {
    let text: ReactNode = inline.text;
    for (const mark of inline.marks) {
      if (mark.type === 'strong') text = <strong>{text}</strong>;
      if (mark.type === 'emphasis') text = <em>{text}</em>;
      if (mark.type === 'code') text = <code>{text}</code>;
      // As in the authoring codec, links remain passive text rather than navigation.
      if (mark.type === 'link') text = <span className="document-link">{text}</span>;
    }
    return <span key={index}>{text}</span>;
  });
  return <>{blocks.map(block => {
    const content = block.content;
    switch (content.type) {
      case 'paragraph': return <p key={block.id}>{inlines(content.inlines)}</p>;
      case 'heading': return createElement(`h${Math.min(6, Math.max(1, content.level))}`, { key: block.id }, inlines(content.inlines));
      case 'code': return <pre key={block.id}><code>{content.text}</code></pre>;
      case 'math': return <div key={block.id}><MathPreview source={content.source} /></div>;
      case 'list': { const items = content.items.map((blocks, index) => <li key={blocks[0]?.id ?? index}><CanonicalReading blocks={blocks} /></li>); return content.ordered ? <ol key={block.id}>{items}</ol> : <ul key={block.id}>{items}</ul>; }
      case 'table': return <table key={block.id}><tbody>{content.rows.map((row, r) => <tr key={r}>{row.map((blocks, c) => <td key={blocks[0]?.id ?? c}><CanonicalReading blocks={blocks} /></td>)}</tr>)}</tbody></table>;
      case 'media': return <figure key={block.id}><p>Preserved attachment: {content.alt || content.mime}</p><figcaption>{content.caption}</figcaption></figure>;
      case 'unsupported': return <aside key={block.id}><p>Unsupported content preserved: {content.reason}</p><pre>{content.original}</pre><SourceLocation source={content.source} /></aside>;
    }
  })}</>;
}

function BlockFields({ block, onChange, readOnly }: { block: Block; onChange: (content: BlockContent) => void; readOnly: boolean }) {
  const content = block.content;
  switch (content.type) {
    case 'paragraph': return <p>Paragraph text and marks can be edited in the document above.</p>;
    case 'heading': return <label>Heading level <select disabled={readOnly} value={content.level} onChange={event => onChange({ ...content, level: Number(event.target.value) })}>{[1, 2, 3, 4, 5, 6].map(level => <option key={level}>{level}</option>)}</select></label>;
    case 'code': return <CodeFields content={content} onChange={onChange} readOnly={readOnly} />;
    case 'math': return <><label>Formula source <textarea disabled={readOnly} value={content.source} onChange={event => onChange({ ...content, source: event.target.value })} /></label><MathPreview source={content.source} /></>;
    case 'media': return <><p>Preserved local attachment ({content.mime}).</p><label>Media caption <input disabled={readOnly} value={content.caption} onChange={event => onChange({ ...content, caption: event.target.value })} /></label><label>Media alternative text <input disabled={readOnly} value={content.alt} onChange={event => onChange({ ...content, alt: event.target.value })} /></label></>;
    case 'unsupported': return <div role="note"><p>Unsupported content preserved: {content.reason}</p><pre>{content.original}</pre><SourceLocation source={content.source} /></div>;
    case 'list': return <ListFields content={content} onChange={onChange} readOnly={readOnly} />;
    case 'table': return <TableFields content={content} onChange={onChange} readOnly={readOnly} />;
  }
}

function ListFields({ content, onChange, readOnly }: { content: Extract<BlockContent, { type: 'list' }>; onChange: (content: BlockContent) => void; readOnly: boolean }) {
  return <fieldset><legend>List items</legend><label><input type="checkbox" disabled={readOnly} checked={content.ordered} onChange={event => onChange({ ...content, ordered: event.target.checked })} />Numbered list</label>
      {content.items.map((blocks, index) => <div key={blocks[0].id}><DocumentEditor label={`List item ${index + 1}`} document={{ schemaVersion: 1, blocks }} readOnly={readOnly} onChange={document => onChange({ ...content, items: content.items.map((item, i) => i === index ? document.blocks : item) })} /><button type="button" disabled={readOnly || content.items.length === 1} onClick={() => onChange({ ...content, items: content.items.filter((_, i) => i !== index) })}>Remove list item {index + 1}</button></div>)}
      <button type="button" disabled={readOnly} onClick={() => onChange({ ...content, items: [...content.items, [newBlock('paragraph')]] })}>Add list item</button></fieldset>;
}

function TableFields({ content, onChange, readOnly }: { content: Extract<BlockContent, { type: 'table' }>; onChange: (content: BlockContent) => void; readOnly: boolean }) {
  return <fieldset><legend>Table cells</legend>
      {content.rows.map((row, r) => <fieldset key={row[0][0].id}><legend>Row {r + 1}</legend>{row.map((blocks, c) => <DocumentEditor key={blocks[0].id} label={`Table row ${r + 1} cell ${c + 1}`} document={{ schemaVersion: 1, blocks }} readOnly={readOnly} onChange={document => onChange({ ...content, rows: content.rows.map((cells, ri) => ri === r ? cells.map((cell, ci) => ci === c ? document.blocks : cell) : cells) })} />)}<button type="button" disabled={readOnly || content.rows.length === 1} onClick={() => onChange({ ...content, rows: content.rows.filter((_, i) => i !== r) })}>Remove table row {r + 1}</button></fieldset>)}
      <button type="button" disabled={readOnly} onClick={() => onChange({ ...content, rows: [...content.rows, content.rows[0].map(() => [newBlock('paragraph')])] })}>Add table row</button>
      <button type="button" disabled={readOnly} onClick={() => onChange({ ...content, rows: content.rows.map(row => [...row, [newBlock('paragraph')]]) })}>Add table column</button>
      <button type="button" disabled={readOnly || content.rows[0].length === 1} onClick={() => onChange({ ...content, rows: content.rows.map(row => row.slice(0, -1)) })}>Remove last table column</button>
    </fieldset>;
}

function CodeFields({ content, onChange, readOnly }: { content: Extract<BlockContent, { type: 'code' }>; onChange: (content: BlockContent) => void; readOnly: boolean }) {
  return <label>Code language <input disabled={readOnly} value={content.language ?? ''} onChange={event => onChange({ ...content, language: event.target.value || null })} /></label>;
}

function SourceLocation({ source }: { source: Extract<BlockContent, { type: 'unsupported' }>['source'] }) {
  return source && <p>Source location: {source.locator}</p>;
}
