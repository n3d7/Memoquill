import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { expect, it, vi } from 'vitest';
import type { ReadingPreferences } from '../application/workspaceTypes';
import type { ReadingController } from '../useReadingPreferences';
import { SettingsPanel, useSettings } from './SettingsPanel';
import { WorkspaceSettingsProvider } from './WorkspaceSettingsProvider';
import { emptyLayer, type SettingsPort } from './types';
function Harness({ save = vi.fn().mockResolvedValue('Reading preferences saved.'), initial = { theme: 'system', textScale: 100, customization: emptyLayer() } }: { save?: (next?: ReadingPreferences) => Promise<string>; initial?: ReadingPreferences }) {
  const [preferences, setPreferences] = useState(initial);
  const reading = { preferences, setPreferences, loaded: true, working: false, retry: false, recovery: false, save: async (next?: ReadingPreferences) => { const result: string = await save(next); if (result.startsWith('Reading preferences saved.')) setPreferences(next ?? preferences); return result; } } as ReadingController;
  const [subject, setSubject] = useState('');
  const settings = useSettings(reading, subject);
  return <><output aria-label="Effective appearance">{settings.effective.theme}/{settings.effective.textScale}</output><button onClick={() => setSubject('one')}>Choose subject one</button><button onClick={() => setSubject('two')}>Choose subject two</button><SettingsPanel reading={reading} settings={settings} subjects={{ one: { title: 'First subject' }, two: { title: 'Second subject' } }} /></>;
}
it('saves course notes, applies temporary overrides and resets one layer without changing saved history', async () => {
  const save = vi.fn().mockResolvedValue('Reading preferences saved.'); render(<Harness save={save} />);
  fireEvent.click(screen.getByText('Goals, focus and inherited settings'));
  fireEvent.click(screen.getByLabelText('Override learning goal'));
  fireEvent.change(screen.getByLabelText('course learning goal'), { target: { value: 'Explain without help' } });
  expect(screen.getByLabelText('Settings scope')).toBeDisabled();
  fireEvent.click(screen.getByRole('button', { name: 'Save settings layer' }));
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0][0].customization.overrides.goal).toBe('Explain without help');
  await waitFor(() => expect(screen.getByLabelText('Settings scope')).toBeEnabled());
  fireEvent.change(screen.getByLabelText('Settings scope'), { target: { value: 'session' } });
  fireEvent.click(screen.getByLabelText('Override current focus')); fireEvent.change(screen.getByLabelText('session current focus'), { target: { value: 'Temporary focus' } });
  fireEvent.click(screen.getByRole('button', { name: 'Apply temporary settings' }));
  await screen.findByText(/Temporary session settings applied/);
  expect(within(screen.getByLabelText('Effective settings')).getByText('Temporary focus · session')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Reset this settings layer' }));
  expect(within(screen.getByLabelText('Effective settings')).getByText('Temporary focus · session')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Apply temporary settings' }));
  await waitFor(() => expect(within(screen.getByLabelText('Effective settings')).queryByText('Temporary focus · session')).not.toBeInTheDocument());
  expect(within(screen.getByLabelText('Effective settings')).getByText('Explain without help · course')).toBeInTheDocument(); expect(save).toHaveBeenCalledOnce();
});
it('imports only supported settings and keeps failed saves editable', async () => {
  const save = vi.fn().mockResolvedValue('Storage is full.'); render(<Harness save={save} />); fireEvent.click(screen.getByText('Goals, focus and inherited settings'));
  fireEvent.change(screen.getByLabelText('Settings JSON'), { target: { value: '{"contractVersion":2,"overrides":{}}' } }); fireEvent.click(screen.getByText('Prepare imported settings')); expect(screen.getByText(/Invalid settings JSON/)).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Settings JSON'), { target: { value: '{"contractVersion":1,"overrides":{"goal":"Keep my draft"}}' } }); fireEvent.click(screen.getByText('Prepare imported settings')); expect(save).not.toHaveBeenCalled();
  fireEvent.click(screen.getByText('Save settings layer')); await screen.findByText('Storage is full.'); expect(screen.getByLabelText('course learning goal')).toHaveValue('Keep my draft'); expect(screen.getByLabelText('Settings scope')).toBeDisabled();
  fireEvent.click(screen.getByText('Export settings JSON')); expect(JSON.parse((screen.getByLabelText('Settings JSON') as HTMLTextAreaElement).value)).toEqual({ contractVersion: 1, overrides: { goal: 'Keep my draft' } });
});
it('propagates workspace defaults while course overrides stay isolated and retries the exact uncertain request', async () => {
  const read = vi.fn().mockResolvedValue({ revision: 0, settings: emptyLayer() });
  const save = vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async request => ({ revision: 1, settings: request.settings }));
  const port: SettingsPort = { read, save }; render(<WorkspaceSettingsProvider port={port}><Harness /></WorkspaceSettingsProvider>);
  fireEvent.click(screen.getByText('Goals, focus and inherited settings')); await waitFor(() => expect(screen.getByLabelText('Settings scope')).toBeEnabled());
  fireEvent.change(screen.getByLabelText('Settings scope'), { target: { value: 'workspace' } }); fireEvent.click(screen.getByLabelText('Override appearance')); fireEvent.change(screen.getByLabelText('workspace appearance'), { target: { value: 'dark' } });
  fireEvent.click(screen.getByText('Save settings layer')); await screen.findByText('Retry workspace settings save'); fireEvent.click(screen.getByText('Retry workspace settings save'));
  await waitFor(() => expect(screen.getByLabelText('Effective appearance')).toHaveTextContent('dark/100')); expect(save.mock.calls[1][0]).toEqual(save.mock.calls[0][0]); expect(save.mock.calls[0][0].expectedRevision).toBe(0);
});

it('retrying shared defaults from another course keeps that course draft intact', async () => {
  const read = vi.fn().mockResolvedValue({ revision: 0, settings: emptyLayer() });
  const save = vi.fn().mockRejectedValueOnce({ code: 'outcome_unconfirmed' }).mockImplementation(async request => ({ revision: 1, settings: request.settings }));
  render(<WorkspaceSettingsProvider port={{ read, save }}><section aria-label="Course A"><Harness /></section><section aria-label="Course B"><Harness /></section></WorkspaceSettingsProvider>);
  const a = within(screen.getByRole('region', { name: 'Course A' })); const b = within(screen.getByRole('region', { name: 'Course B' }));
  fireEvent.click(a.getByText('Goals, focus and inherited settings')); fireEvent.click(b.getByText('Goals, focus and inherited settings'));
  await waitFor(() => expect(a.getByLabelText('Settings scope')).toBeEnabled());
  fireEvent.click(b.getByLabelText('Override learning goal')); fireEvent.change(b.getByLabelText('course learning goal'), { target: { value: 'Unsaved B goal' } });
  fireEvent.change(a.getByLabelText('Settings scope'), { target: { value: 'workspace' } }); fireEvent.click(a.getByLabelText('Override current focus')); fireEvent.change(a.getByLabelText('workspace current focus'), { target: { value: 'Shared focus' } });
  fireEvent.click(a.getByText('Save settings layer')); await b.findByText('Retry workspace settings save'); fireEvent.click(b.getByText('Retry workspace settings save'));
  await waitFor(() => expect(b.getByLabelText('course learning goal')).toBeEnabled()); expect(b.getByLabelText('course learning goal')).toHaveValue('Unsaved B goal'); expect(b.getByLabelText('Settings scope')).toBeDisabled();
  expect(save.mock.calls[1][0]).toEqual(save.mock.calls[0][0]);
});

it('keeps an unsaved subject draft attached to its original owner across selection changes', async () => {
  const save = vi.fn().mockResolvedValue('Reading preferences saved.');
  render(<Harness save={save} initial={{ theme: 'system', textScale: 100, customization: emptyLayer(), subjectSettings: { two: { contractVersion: 1, overrides: { goal: 'Keep second goal' } } } }} />);
  fireEvent.click(screen.getByText('Goals, focus and inherited settings'));
  fireEvent.click(screen.getByText('Choose subject one'));
  fireEvent.change(screen.getByLabelText('Settings scope'), { target: { value: 'subject' } });
  fireEvent.click(screen.getByLabelText('Override learning goal'));
  fireEvent.change(screen.getByLabelText('subject learning goal'), { target: { value: 'First draft goal' } });
  fireEvent.click(screen.getByText('Choose subject two'));
  expect(screen.getByText(/draft still belongs to the original subject/)).toBeInTheDocument();
  expect(screen.getByLabelText('subject learning goal')).toHaveValue('First draft goal');
  fireEvent.click(screen.getByRole('button', { name: 'Save settings layer' }));
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0][0].subjectSettings.one.overrides.goal).toBe('First draft goal');
  expect(save.mock.calls[0][0].subjectSettings.two.overrides.goal).toBe('Keep second goal');
  await waitFor(() => expect(screen.getByLabelText('subject learning goal')).toHaveValue('Keep second goal'));
});
it('resets only the selected subject layer, retaining course settings and other subjects', async () => {
  const save = vi.fn().mockResolvedValue('Reading preferences saved.');
  render(<Harness save={save} initial={{ theme: 'dark', textScale: 125, customization: { contractVersion: 1, overrides: { goal: 'Course goal' } }, subjectSettings: { one: { contractVersion: 1, overrides: { focus: 'First focus' } }, two: emptyLayer() } }} />);
  fireEvent.click(screen.getByText('Goals, focus and inherited settings')); fireEvent.click(screen.getByText('Choose subject one'));
  fireEvent.change(screen.getByLabelText('Settings scope'), { target: { value: 'subject' } });
  fireEvent.click(screen.getByRole('button', { name: 'Reset this settings layer' }));
  expect(save).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Save settings layer' }));
  await waitFor(() => expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0][0].subjectSettings).toEqual({ two: emptyLayer() });
  expect(save.mock.calls[0][0].customization.overrides.goal).toBe('Course goal');
});
it('blocks conflicting keyboard bindings, stages reset and keeps settings-layer edits separate', async () => {
  const save = vi.fn().mockResolvedValue('Reading preferences saved.'); render(<Harness save={save} />);
  fireEvent.click(screen.getByText('Goals, focus and inherited settings'));
  fireEvent.change(screen.getByLabelText('Settings shortcut'), { target: { value: 'Ctrl+Alt+Shift+S' } });
  fireEvent.change(screen.getByLabelText('Search shortcut'), { target: { value: 'Ctrl+Alt+Shift+S' } });
  fireEvent.click(screen.getByText('Save keyboard preferences'));
  expect(await screen.findByText(/Use distinct/)).toBeInTheDocument(); expect(save).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText('Search shortcut'), { target: { value: 'Ctrl+Alt+Shift+F' } });
  fireEvent.click(screen.getByText('Save keyboard preferences')); await waitFor(() => expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0][0].shortcuts.bindings.focusSearch).toBe('Ctrl+Alt+Shift+F');
  fireEvent.click(screen.getByText('Reset keyboard preferences')); expect(save).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByText('Save keyboard preferences')); await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(Object.values(save.mock.calls[1][0].shortcuts.bindings)).toEqual([null, null, null]);
  expect(save.mock.calls[1][0].customization).toEqual(emptyLayer());
});
