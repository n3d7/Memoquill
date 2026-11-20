import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it, vi } from 'vitest';
import { emptyShortcuts, parseShortcuts, useKeyboardShortcuts, type ShortcutPreferences } from './shortcuts';

it('validates versioned shortcut imports and rejects reserved, duplicate and unknown bindings', () => {
  for (const value of [null, { contractVersion: 2, bindings: {} }, { contractVersion: 1, bindings: { unknown: 'Ctrl+Alt+Shift+S' } }, { contractVersion: 1, bindings: { openSettings: 'Ctrl+Alt+Shift+R' } }, { contractVersion: 1, bindings: { openSettings: 'Ctrl+W' } }, { contractVersion: 1, bindings: { openSettings: 'Ctrl+Alt+Shift+S', focusSearch: 'Ctrl+Alt+Shift+S' } }]) expect(() => parseShortcuts(JSON.stringify(value))).toThrow();
  expect(parseShortcuts('{"contractVersion":1,"bindings":{}}')).toEqual(emptyShortcuts());
});
it('handles only active explicit actions and preserves editable fields, composition and ordinary keys', () => {
  const open = vi.fn(() => true); const preferences = emptyShortcuts(); preferences.bindings.openSettings = 'Ctrl+Alt+Shift+S';
  function Harness({ prefs }: { prefs: ShortcutPreferences | null }) {
    useKeyboardShortcuts(prefs, { openSettings: open, focusSearch: () => false, focusWorkspace: () => false });
    return <><button>Ordinary action</button><input aria-label="Editor" /><div contentEditable aria-label="Rich editor" /></>;
  }
  const { rerender } = render(<Harness prefs={preferences} />);
  const chord = { code: 'KeyS', key: 'ы', ctrlKey: true, altKey: true, shiftKey: true, cancelable: true };
  fireEvent.keyDown(screen.getByLabelText('Editor'), chord);
  fireEvent.keyDown(screen.getByLabelText('Rich editor'), chord);
  fireEvent.keyDown(document, { ...chord, isComposing: true });
  fireEvent.keyDown(document, { ...chord, repeat: true });
  fireEvent.keyDown(document, { ...chord, altKey: false });
  expect(open).not.toHaveBeenCalled();
  const event = new KeyboardEvent('keydown', chord); document.dispatchEvent(event);
  expect(open).toHaveBeenCalledOnce(); expect(event.defaultPrevented).toBe(true);
  rerender(<Harness prefs={null} />); fireEvent.keyDown(document, chord); expect(open).toHaveBeenCalledOnce();
});
