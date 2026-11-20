import { KeyboardPreferences } from './KeyboardPreferences';
import { useEffect, useState, type Ref } from 'react';
import { useApplicationActive } from '../application/ApplicationProvider';
import type { ReadingController } from '../useReadingPreferences';
import { courseLayer, emptyLayer, parseSettings, resolveSettings, settingKeys, type EffectiveSettings, type SettingsLayer, type SettingKey } from './types';
import { useWorkspaceDefaults } from './WorkspaceSettingsProvider';
export function useSettings(reading: ReadingController, subjectId = '') {
  const workspace = useWorkspaceDefaults();
  const active = useApplicationActive();
  const [session, setSession] = useState(emptyLayer);
  const resolved = resolveSettings(workspace?.settings ?? emptyLayer(), courseLayer(reading.preferences), session, reading.preferences.subjectSettings?.[subjectId] ?? emptyLayer());
  useEffect(() => { if (active) document.documentElement.style.fontSize = `${resolved.effective.textScale}%`; }, [resolved.effective.textScale, active]);
  return { workspace, session, setSession, subjectId, ...resolved };
}
type Controller = ReturnType<typeof useSettings>;
export function SettingsPanel({ reading, settings, detailsRef, subjects = {} }: { reading: ReadingController; settings: Controller; detailsRef?: Ref<HTMLDetailsElement>; subjects?: Record<string, { title: string }> }) {
  const [scope, setScope] = useState<'workspace' | 'course' | 'subject' | 'session'>('course');
  const [draft, setDraft] = useState<SettingsLayer>(() => courseLayer(reading.preferences));
  const [json, setJson] = useState(''); const [notice, setNotice] = useState(''); const [dirty, setDirty] = useState(false);
  const [subjectOwner, setSubjectOwner] = useState(settings.subjectId);
  const owner = dirty ? subjectOwner : settings.subjectId;
  const workspace = settings.workspace;
  const source = scope === 'workspace' ? workspace?.settings ?? emptyLayer() : scope === 'course' ? courseLayer(reading.preferences) : scope === 'subject' ? reading.preferences.subjectSettings?.[owner] ?? emptyLayer() : settings.session;
  const signature = JSON.stringify(source);
  useEffect(() => { if (!dirty) setDraft(JSON.parse(signature) as SettingsLayer); }, [signature, dirty]);
  const blocked = reading.working || reading.retry || !reading.loaded || reading.recovery || !!workspace?.busy || !!workspace?.pending || (!!workspace?.available && !workspace.loaded);
  function markDirty() { if (!dirty) setSubjectOwner(settings.subjectId); setDirty(true); }
  function edit(key: SettingKey, value: string) {
    markDirty(); setDraft(current => ({ ...current, overrides: { ...current.overrides, [key]: key === 'textScale' ? Number(value) : value } }));
  }
  function inherit(key: SettingKey) {
    markDirty(); setDraft(current => { const overrides = { ...current.overrides }; delete overrides[key]; return { ...current, overrides }; });
  }
  async function save() {
    // UI parsing and native validation enforce the same versioned/bounded contract.
    let validated: SettingsLayer;
    try { validated = parseSettings(JSON.stringify(draft)); } catch { setNotice('Use supported values and at most 2048 characters per note.'); return; }
    if (scope === 'workspace') { if (!await workspace?.save(validated)) return; }
    else if (scope === 'course') { const message = await reading.save({ ...reading.preferences, customization: validated }); setNotice(message); if (!message.startsWith('Reading preferences saved.')) return; }
    else if (scope === 'subject') {
      if (!owner) { setNotice('Select a subject before saving its overrides.'); return; }
      const subjectSettings = { ...reading.preferences.subjectSettings };
      if (Object.keys(validated.overrides).length) subjectSettings[owner] = validated; else delete subjectSettings[owner];
      const message = await reading.save({ ...reading.preferences, subjectSettings: Object.keys(subjectSettings).length ? subjectSettings : null });
      setNotice(message); if (!message.startsWith('Reading preferences saved.')) return;
    }
    else { settings.setSession(validated); setNotice('Temporary session settings applied. They last until this course view closes or the app restarts.'); }
    setDirty(false);
  }
  const labels: Record<SettingKey, string> = { theme: 'Appearance', textScale: 'Text size', goal: 'Learning goal', focus: 'Current focus' };
  const parent = scope === 'workspace' ? resolveSettings(emptyLayer(), emptyLayer(), emptyLayer()).effective : scope === 'course' ? resolveSettings(workspace?.settings ?? emptyLayer(), emptyLayer(), emptyLayer()).effective : resolveSettings(workspace?.settings ?? emptyLayer(), courseLayer(reading.preferences), emptyLayer(), scope === 'session' ? reading.preferences.subjectSettings?.[settings.subjectId] ?? emptyLayer() : emptyLayer()).effective;
  return <details ref={detailsRef}><summary>Goals, focus and inherited settings</summary>
    <p>Workspace defaults → course overrides → selected subject overrides → temporary session overrides. Notes express your intentions; they do not measure competence or change scheduling. Existing subject goals remain separate.</p>
    <label>Settings scope<select value={scope} disabled={blocked || dirty} onChange={event => { setScope(event.target.value as typeof scope); setSubjectOwner(settings.subjectId); setNotice(''); setJson(''); }}>
      {workspace?.available && <option value="workspace">Workspace defaults</option>}<option value="course">Course overrides</option><option value="subject" disabled={!settings.subjectId}>Subject overrides</option><option value="session">Temporary session overrides</option>
    </select></label>
    {scope === 'subject' && <p>Editing subject settings: {subjects[owner]?.title ?? 'Selected subject'}. {dirty && owner !== settings.subjectId && 'The selection changed; this draft still belongs to the original subject.'}</p>}
    {dirty && <button disabled={blocked} onClick={() => { setDraft(source); setDirty(false); }}>Discard settings edits</button>}
    <fieldset disabled={blocked || (scope === 'subject' && !owner)}><legend>{scope === 'workspace' ? 'Workspace defaults' : scope === 'course' ? 'Course overrides' : scope === 'subject' ? 'Subject overrides' : 'Temporary session overrides'}</legend>
      {settingKeys.map(key => <div key={key}><label><input type="checkbox" checked={Object.hasOwn(draft.overrides, key)} onChange={event => { if (event.target.checked) edit(key, String(parent[key])); else inherit(key); }} />Override {labels[key].toLowerCase()}</label>
        {Object.hasOwn(draft.overrides, key) ? <SettingInput name={`${scope} ${labels[key].toLowerCase()}`} field={key} value={draft.overrides[key] ?? ''} edit={value => edit(key, value)} /> : <p>{labels[key]}: inherits {String(parent[key]) || '(empty)'}</p>}
      </div>)}
      <button onClick={() => void save()}>{scope === 'session' ? 'Apply temporary settings' : 'Save settings layer'}</button>
      <button onClick={() => { setDraft(emptyLayer()); markDirty(); setNotice('Layer reset prepared. Save or apply to confirm; learning data is retained.'); }}>Reset this settings layer</button>
    </fieldset>
    {reading.retry && <button disabled={reading.working} onClick={() => void reading.save().then(message => { setNotice(message); if (message.startsWith('Reading preferences saved.') && (scope === 'course' && JSON.stringify(draft) === JSON.stringify(courseLayer(reading.pendingPreferences ?? reading.preferences)) || scope === 'subject' && JSON.stringify(draft) === JSON.stringify(reading.pendingPreferences?.subjectSettings?.[owner] ?? emptyLayer()))) setDirty(false); })}>Retry course settings save</button>}
    {workspace?.available && <><button disabled={workspace.busy} onClick={() => void workspace.reload()}>Reload workspace defaults</button>{workspace.pending && <button disabled={workspace.busy} onClick={() => void workspace.save(draft).then(saved => { if (saved && scope === 'workspace' && JSON.stringify(draft) === JSON.stringify(workspace.pendingSettings)) setDirty(false); })}>Retry workspace settings save</button>}<p role="status">{workspace.message}</p></>}
    <p role="status">{notice}</p>
    <dl aria-label="Effective settings">{settingKeys.map(key => <div key={key}><dt>{labels[key]}</dt><dd>{String(settings.effective[key]) || '(empty)'} · {settings.sources[key]}</dd></div>)}</dl>
    <fieldset disabled={blocked || (scope === 'subject' && !owner)}><legend>Settings JSON transfer</legend><p>Export or paste this layer only. It may contain private goal/focus notes. It contains no navigation or learning history. Import prepares edits; save or apply explicitly.</p>
      <button onClick={() => setJson(JSON.stringify(draft, null, 2))}>Export settings JSON</button><label>Settings JSON<textarea value={json} maxLength={32768} onChange={event => setJson(event.target.value)} /></label>
      <button onClick={() => { try { setDraft(parseSettings(json)); markDirty(); setNotice('Imported settings prepared. Save or apply to confirm.'); } catch { setNotice('Invalid settings JSON. Supported version: 1; unknown fields and invalid values are rejected.'); } }}>Prepare imported settings</button>
    </fieldset>
    <KeyboardPreferences reading={reading} />
  </details>;
}
function SettingInput({ name, field, value, edit }: { name: string; field: SettingKey; value: EffectiveSettings[SettingKey]; edit: (value: string) => void }) {
  if (field === 'theme') return <label>{name}<select value={value} onChange={event => edit(event.target.value)}>{['system', 'light', 'dark'].map(v => <option key={v}>{v}</option>)}</select></label>;
  if (field === 'textScale') return <label>{name}<select value={value} onChange={event => edit(event.target.value)}>{[100, 125, 150, 200].map(v => <option key={v} value={v}>{v}%</option>)}</select></label>;
  return <label>{name}<textarea value={value} maxLength={2048} onChange={event => edit(event.target.value)} /></label>;
}
