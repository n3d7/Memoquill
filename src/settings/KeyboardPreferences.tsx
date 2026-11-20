import { useEffect, useState } from 'react';
import type { ReadingController } from '../useReadingPreferences';
import { emptyShortcuts, parseShortcuts, shortcutActions, shortcutLabels, type ShortcutPreferences } from './shortcuts';

export function KeyboardPreferences({ reading }: { reading: ReadingController }) {
  const saved = reading.preferences.shortcuts ?? emptyShortcuts(); const signature = JSON.stringify(saved);
  const [draft, setDraft] = useState<ShortcutPreferences>(saved); const [dirty, setDirty] = useState(false);
  const [json, setJson] = useState(''); const [notice, setNotice] = useState('');
  const draftSignature = JSON.stringify(draft);
  useEffect(() => { if (!dirty && signature !== draftSignature) setDraft(JSON.parse(signature) as ShortcutPreferences); else if (dirty && !reading.retry && signature === draftSignature) setDirty(false); }, [signature, dirty, draftSignature, reading.retry]);
  const blocked = !reading.loaded || reading.working || reading.retry || reading.recovery;
  async function save(retry = false) {
    try {
      const validated = parseShortcuts(JSON.stringify(draft));
      const message = await reading.save(retry ? undefined : { ...reading.preferences, shortcuts: validated });
      setNotice(message); if (message.startsWith('Reading preferences saved.')) setDirty(false);
    } catch (reason) { setNotice(reason instanceof Error ? reason.message : 'Invalid shortcuts.'); }
  }
  return <section aria-label="Keyboard preferences"><h3>Keyboard preferences</h3>
    <p>Optional shortcuts for this course view; buttons and Settings remain available. Use Ctrl+Alt+Shift and a physical Latin-letter key. R is reserved. Duplicate bindings are rejected. Desktop or custom OS shortcuts may intercept keys; disable or change a binding if that occurs. Shortcuts do not run while typing in a field.</p>
    <fieldset disabled={blocked}><legend>Course keyboard bindings</legend>
      {shortcutActions.map(action => <label key={action}>{shortcutLabels[action]}<input value={draft.bindings[action] ?? ''} maxLength={16} placeholder="Disabled" onChange={e => { setDraft({ ...draft, bindings: { ...draft.bindings, [action]: e.target.value || null } }); setDirty(true); }} /></label>)}
      <button onClick={() => void save()}>Save keyboard preferences</button>
      <button onClick={() => { setDraft(emptyShortcuts()); setDirty(true); setNotice('Keyboard reset prepared. Save to confirm; content and other settings are retained.'); }}>Reset keyboard preferences</button>
      {dirty && <button onClick={() => { setDraft(saved); setDirty(false); }}>Discard keyboard edits</button>}
      <button onClick={() => setJson(JSON.stringify(draft, null, 2))}>Export keyboard JSON</button>
      <label>Keyboard JSON<textarea maxLength={2048} value={json} onChange={e => setJson(e.target.value)} /></label>
      <button onClick={() => { try { setDraft(parseShortcuts(json)); setDirty(true); setNotice('Keyboard import prepared. Save to confirm.'); } catch (reason) { setNotice(reason instanceof Error ? reason.message : 'Invalid shortcuts.'); } }}>Prepare keyboard import</button>
    </fieldset>
    {reading.retry && JSON.stringify(reading.pendingPreferences?.shortcuts) === JSON.stringify(draft) && <button disabled={reading.working} onClick={() => void save(true)}>Retry keyboard save</button>}
    <p role="status">{notice}</p>
  </section>;
}
