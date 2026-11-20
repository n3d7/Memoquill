import { useId } from 'react';
import { useResultDialog } from './useResultDialog';
import type { ReadingController } from './useReadingPreferences';
import type { TextScale, Theme } from './workspaceBridge';

const themes: Theme[] = ['system', 'light', 'dark'];
const scales: TextScale[] = [100, 125, 150, 200];

export function ReadingPreferencesPanel({ reading, effective }: { reading: ReadingController; effective?: { theme: Theme; textScale: TextScale } }) {
  const messageId = useId();
  const result = useResultDialog();
  const current = effective ?? reading.preferences;
  function appearance(part: Partial<Pick<typeof reading.preferences, 'theme' | 'textScale'>>) {
    reading.setPreferences({ ...reading.preferences, ...part, ...(reading.preferences.customization ? { customization: { contractVersion: 1, overrides: { ...reading.preferences.customization.overrides, ...part } } } : {}) });
  }
  async function show(action: () => Promise<string>, origin: HTMLButtonElement) {
    const message = await action();result.show(message, origin);
  }
  return <section className="reading-preferences" aria-label="Local workspace">
    <fieldset className="appearance" aria-describedby={messageId} disabled={reading.working}>
      <legend>Reading preferences</legend>
      <label>Appearance<select value={current.theme} disabled={!reading.loaded || reading.retry}
        onChange={event => {
          const theme = themes.find(value => value === event.target.value);
          if (theme) appearance({ theme });
        }}>
        <option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option>
      </select></label>
      <label>Text size<select value={current.textScale} disabled={!reading.loaded || reading.retry}
        onChange={event => {
          const textScale = scales.find(value => value === Number(event.target.value));
          if (textScale) appearance({ textScale });
        }}>
        {scales.map(value => <option key={value} value={value}>{value}%</option>)}
      </select></label>
      <button disabled={!reading.loaded || reading.recovery}
        onClick={event => void show(reading.save, event.currentTarget)}>
        {reading.retry ? 'Retry saving preferences' : 'Save reading preferences'}
      </button>
      <button disabled={!reading.loaded || reading.retry} onClick={() => appearance({ theme: 'system', textScale: 100 })}>Use default reading appearance</button>
      <button onClick={event => void show(reading.reload, event.currentTarget)}>
        {reading.loaded ? 'Reload saved preferences' : 'Retry workspace'}
      </button>
      <button onClick={event => result.show(reading.message, event.currentTarget)}>Read workspace status</button>
    </fieldset>
    <p id={messageId} aria-live="polite" aria-atomic="true">{reading.working ? 'Working with local workspace…' : reading.message}</p>
    {result.dialog}
  </section>;
}
