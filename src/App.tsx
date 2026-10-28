import { useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { checkDraft } from './bridge';
import type { QualificationDraft } from './bridge';
import { ProseEditor } from './ProseEditor';
import { Preview } from './Preview';
import { usePickerFocus } from './usePickerFocus';
import { useResultDialog } from './useResultDialog';

const initialNotes = 'Explain a relationship in your own words.\nRecall café 世界 — then compare your answer with the source.\nFor a practical procedure, describe the ordered steps and check your performance separately.';

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return typeof error === 'string' ? error : 'The action could not be completed. Your edits are still here.';
}

export function App() {
  const [title, setTitle] = useState('Learning material fixture');
  const [prose, setProse] = useState(initialNotes);
  const [formula, setFormula] = useState('E=\\hbar\\omega');
  const [code, setCode] = useState('def double(x):\n    return x * 2');
  const [preview, setPreview] = useState(false);
  const [theme, setTheme] = useState('system');
  const [scale, setScale] = useState('100');
  const [status, setStatus] = useState('Ready. No files have been read.');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const previewTrigger = useRef<HTMLButtonElement>(null);
  const result = useResultDialog();
  const pickerFocus = usePickerFocus(busy);
  const draft: QualificationDraft = { schemaVersion: 1, title, prose, formula, code };

  async function runAction(action: () => Promise<string>) {
    setBusy(true);
    setError(null);
    try {
      const message = await action();
      setStatus(message);
      return message;
    }
    catch (reason) {
      const message = errorMessage(reason);
      setError(message);
      return message;
    }
    finally { setBusy(false); }
  }

  async function validate(origin: HTMLButtonElement) {
    const message = await runAction(async () => {
      const result = await checkDraft(draft);
      return `Draft checked: ${result.title}. ${result.proseCharacters} note characters; ${result.codeLines} code lines. Nothing saved.`;
    });
    result.show(message, origin);
  }

  async function chooseFile() {
    pickerFocus.requestFocus();
    await runAction(async () => {
      const path = await open({ multiple: false, directory: false, title: 'Choose a source (name only; no import)' });
      return path ? `Selected ${path.split('/').pop()}. Contents were not read or executed.` : 'No file selected. Your edits are unchanged.';
    });
  }

  return <div className="app" data-theme={theme}>
    <a className="skip-link" href="#main">Skip to material</a>
    <header><p className="eyebrow">Desktop qualification · M1.1</p><h1>Memoquill</h1>
      <p>Read, edit and check a small learning material fixture.</p>
      <p className="notice">Changes are kept only until this window closes. This fixture has no storage or learning history.</p>
    </header>
    <fieldset className="appearance"><legend>Reading preferences</legend>
      <label>Appearance<select value={theme} onChange={event => setTheme(event.target.value)}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
      <label>Text size<select value={scale} onChange={event => { setScale(event.target.value); document.documentElement.style.fontSize = `${event.target.value}%`; }}>
        {['100', '125', '150', '200'].map(value => <option key={value} value={value}>{value}%</option>)}
      </select></label>
    </fieldset>
    <main id="main" tabIndex={-1}>
      <h2>Learning material</h2>
      <label htmlFor="title">Title</label><input id="title" value={title} onChange={event => setTitle(event.target.value)} />
      <h3>Learning notes</h3><p id="notes-help">Type freely, including multilingual text. Ctrl+B toggles bold; Ctrl+Z undoes. Pasted content is plain text. Formatting stays in this editor; the check and reading preview use plain text.</p>
      <ProseEditor initialText={initialNotes} onChange={setProse} />
      <div className="source-grid">
        <div><label htmlFor="formula">Formula source (LaTeX)</label><textarea id="formula" rows={3} value={formula} onChange={event => setFormula(event.target.value)} />
          <p>Preview includes native MathML and the readable source. Speech support must be checked with the installed reader.</p></div>
        <div><label htmlFor="code">Code source (never executed)</label><textarea id="code" rows={5} value={code} onChange={event => setCode(event.target.value)} spellCheck={false} /></div>
      </div>
      <div className="actions">
        <button ref={previewTrigger} onClick={() => setPreview(true)}>Read preview</button>
        <button disabled={busy} onClick={event => void validate(event.currentTarget)}>Check draft</button>
        <button ref={pickerFocus.trigger} disabled={busy} onClick={chooseFile}>Choose a source file</button>
        <button disabled={busy} onClick={event => result.show(error ?? status, event.currentTarget)}>Read last result</button>
      </div>
      <p id="operation-status" role="status" aria-live="polite" aria-atomic="true">{busy ? 'Working…' : status}</p>
      {error && <p role="alert">{error}</p>}
    </main>
    {preview && <Preview draft={draft} onClose={() => { setPreview(false); previewTrigger.current?.focus(); }} />}
    {result.dialog}
  </div>;
}
