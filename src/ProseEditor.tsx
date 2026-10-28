import { useEffect, useRef } from 'react';
import { Schema } from 'prosemirror-model';
import { EditorState } from 'prosemirror-state';
import { EditorView } from 'prosemirror-view';
import { baseKeymap, toggleMark } from 'prosemirror-commands';
import { history, undo, redo } from 'prosemirror-history';
import { keymap } from 'prosemirror-keymap';
import 'prosemirror-view/style/prosemirror.css';

const schema = new Schema({
  nodes: {
    doc: { content: 'paragraph+' },
    paragraph: { content: 'text*', group: 'block', parseDOM: [{ tag: 'p' }], toDOM: () => ['p', 0] },
    text: { group: 'inline' },
  },
  marks: {
    strong: { parseDOM: [{ tag: 'strong' }, { tag: 'b' }], toDOM: () => ['strong', 0] },
  },
});

export function ProseEditor({ initialText, onChange }: { initialText: string; onChange: (text: string) => void }) {
  const container = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!container.current) return;
    const doc = schema.node('doc', null, initialText.split('\n').map(text =>
      schema.node('paragraph', null, text ? schema.text(text) : undefined)));
    const state = EditorState.create({ doc, plugins: [history(), keymap({ 'Mod-b': toggleMark(schema.marks.strong), 'Mod-z': undo, 'Mod-Shift-z': redo }), keymap(baseKeymap)] });
    const view = new EditorView(container.current, {
      state,
      attributes: { role: 'textbox', 'aria-label': 'Learning notes', 'aria-multiline': 'true', 'aria-describedby': 'notes-help', spellcheck: 'true' },
      dispatchTransaction(transaction) {
        view.updateState(view.state.apply(transaction));
        if (transaction.docChanged) onChange(view.state.doc.textBetween(0, view.state.doc.content.size, '\n'));
      },
      handleDOMEvents: {
        paste(editor, event) {
          event.preventDefault();
          const text = event.clipboardData?.getData('text/plain');
          if (text !== undefined) editor.dispatch(editor.state.tr.insertText(text));
          return true; // Intercept before ProseMirror parses clipboard HTML.
        },
      },
    });
    return () => view.destroy();
  }, [initialText, onChange]);
  return <div ref={container} className="prose-editor" />;
}
