"""Exercise the installed GTK/WebKitGTK application on an isolated AT-SPI bus.

Keyboard events go through Xvfb, or its nested Weston X11 input backend. The
Wayland app itself uses GDK's Wayland backend, not Xwayland. No synthetic DOM
events or direct calls into application internals are used.
"""
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
import pyatspi
from gi.repository import GLib
from ibus_control import select_engine

mode = sys.argv[1]
results = []

def command(*args):
    result = subprocess.run(args, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(f'{args[0]} failed ({result.returncode}): {result.stderr.strip()}')
    return result.stdout.strip()

def nodes(root=None):
    root = pyatspi.Registry.getDesktop(0) if root is None else root
    found = []
    def visit(node, depth=0):
        if depth > 30 or len(found) > 1800:
            return
        try:
            found.append(node)
            for child in node:
                visit(child, depth + 1)
        except GLib.Error:
            pass  # A transient portal/IME application can leave the bus.
    visit(root)
    return found

def find(role, name=None):
    for node in nodes():
        if node.getRoleName() == role and (name is None or node.name == name):
            return node
    raise LookupError(f'Accessible control not found: {role} {name}')

def wait_for(test, description, record=True):
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        try:
            if test():
                if record:
                    results.append({'check': description, 'passed': True})
                return
        except (LookupError, GLib.Error):
            pass
        time.sleep(.1)
    focused = [{'role': node.getRoleName(), 'name': node.name} for node in nodes() if node.getState().contains(pyatspi.STATE_FOCUSED)]
    active_window = command('xdotool', 'getwindowfocus', 'getwindowname')
    results.append({'failed': description, 'focusedControls': focused, 'activeXWindow': active_window})
    capture('failure')
    raise AssertionError(f'{description}; focused controls: {focused}')

def text(node):
    try:
        value = node.queryText().getText(0, -1)
        if '\ufffc' not in value:
            return value
    except NotImplementedError:
        pass
    return '\n'.join(text(child) for child in node)

def selected_source():
    return any('Selected memoquill-example.md' in text(node) for node in nodes() if 'live:polite' in node.getAttributes())

def feedback():
    return next(node for node in nodes() if node.name == 'Material operation status')

def open_enabled():
    return any(node.name == 'Open' and node.getRoleName() in ('button', 'push button') and node.getState().contains(pyatspi.STATE_ENABLED) for node in nodes())

def key(value):
    command('xdotool', 'key', '--clearmodifiers', value)
    time.sleep(.15)

def read_from_focus():
    key('KP_Add')  # Orca Say All: request reading rather than infer notification.
    time.sleep(1)
    key('Control_L')

def edit_field(name):
    node = focus('entry', name)
    key('Return')  # Activate the field from Orca browse mode before typing.
    return node

def read_result(expected):
    focus('button', 'Read last result')
    key('space')
    wait_for(lambda: expected in text(find('dialog', 'Operation result')), f'explicit result contains {expected}')
    wait_for(lambda: find('button', 'Close result').getState().contains(pyatspi.STATE_FOCUSED), 'requested result receives focus')
    key('Escape')
    wait_for(lambda: find('button', 'Read last result').getState().contains(pyatspi.STATE_FOCUSED), 'result dismissal returns to its actual trigger')

def focus(role, name):
    node = find(role, name)
    if not node.queryComponent().grabFocus():
        raise AssertionError(f'Cannot focus {name}')
    wait_for(lambda: find(role, name).getState().contains(pyatspi.STATE_FOCUSED), f'focus settles on {name}', record=False)
    return node

def capture(name):
    import gi
    previous = os.environ.get('GDK_BACKEND')
    os.environ['GDK_BACKEND'] = 'x11'  # Capture the parent surface containing Weston.
    try:
        gi.require_version('Gdk', '3.0')
        from gi.repository import Gdk
        window = Gdk.get_default_root_window()
        image = Gdk.pixbuf_get_from_window(window, 0, 0, window.get_width(), window.get_height())
        image.savev(f'/evidence/{name}-{mode}.png', 'png', [], [])
    finally:
        if previous is None:
            os.environ.pop('GDK_BACKEND', None)
        else:
            os.environ['GDK_BACKEND'] = previous

def storage_restart():
    """Persist preferences through real UI; restart normally in this private profile."""
    import fcntl
    focus('button', 'Save reading preferences')
    key('space')
    wait_for(lambda: 'Reading preferences saved' in text(find('dialog', 'Operation result')), 'native preference save acknowledged')
    key('Escape')
    wait_for(lambda: find('button', 'Save reading preferences').getState().contains(pyatspi.STATE_FOCUSED), 'save result restores its trigger')
    data_root = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share')))
    workspace = data_root / 'io.github.n3d7.memoquill/workspace'
    if mode == 'wayland':
        # Nested Weston has no Alt+F4 close binding. Activate GTK's real close control.
        if not find('button', 'Close').queryAction().doAction(0):
            raise AssertionError('Native window-close action was not accepted')
    else:
        key('alt+F4')
    def released():
        with (workspace / 'workspace.lock').open('rb') as lock:
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                return True
            except BlockingIOError:
                return False
    wait_for(released, 'normal window close drains storage and releases owner lock')
    with Path(f'/evidence/restart-{mode}.log').open('w') as output:
        child_env = {**os.environ, 'GDK_BACKEND': mode, 'WAYLAND_DEBUG': '1'}
        child = subprocess.Popen(['memoquill'], env=child_env, stdout=output, stderr=subprocess.STDOUT)
        try:
            if mode == 'wayland':
                wait_for(lambda: 'wl_display' in Path('/evidence/restart-wayland.log').read_text(), 'restarted app uses native Wayland protocol')
            wait_for(lambda: any(node.name == 'Dark' and node.getState().contains(pyatspi.STATE_SELECTED) for node in nodes(find('combo box', 'Appearance'))), 'saved appearance restored after native restart')
            wait_for(lambda: any(node.name == '200%' and node.getState().contains(pyatspi.STATE_SELECTED) for node in nodes(find('combo box', 'Text size'))), 'saved text scale restored after native restart')
            wait_for(lambda: text(find('entry', 'Title')) == 'Learning material fixture', 'material fixture remains explicitly ephemeral')
            focus('button', 'Read workspace status'); key('space')
            wait_for(lambda: 'Reading preferences are saved locally' in text(find('dialog', 'Operation result')), 'native workspace status is available on request')
            capture('storage-restored')
            key('Escape')
        finally:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill(); child.wait()

try:
    if mode == 'wayland':
        output = Path('/evidence/weston.log').read_text()
        match = re.search(r'window id (\d+)', output)
        if match is None:
            raise RuntimeError('Nested Weston did not report its X11 input surface')
        windows = [match.group(1)]
        if 'wl_display' not in Path('/evidence/app-wayland.log').read_text():
            raise AssertionError('App has no native Wayland protocol evidence')
        results.append({'check': 'native Wayland protocol connection', 'passed': True})
    else:
        windows = command('xdotool', 'search', '--name', 'Memoquill').splitlines()
    command('xdotool', 'windowfocus', windows[-1])
    title = edit_field('Title')
    key('ctrl+a')
    command('xdotool', 'type', '--clearmodifiers', 'Quantum physics')
    wait_for(lambda: text(find('entry', 'Title')) == 'Quantum physics', 'native keyboard editing')
    capture('initial')

    focus('button', 'Check draft')
    key('space')
    wait_for(lambda: 'Draft checked: Quantum physics' in text(find('dialog', 'Operation result')), 'packaged typed IPC and explicit result')
    wait_for(lambda: find('button', 'Close result').getState().contains(pyatspi.STATE_FOCUSED), 'result dialog initial focus')
    capture('result')
    key('shift+Tab')
    wait_for(lambda: find('button', 'Close result').getState().contains(pyatspi.STATE_FOCUSED), 'result dialog reverse Tab containment')
    key('Escape')
    wait_for(lambda: find('button', 'Check draft').getState().contains(pyatspi.STATE_FOCUSED), 'result Escape returns to check trigger')

    edit_field('Title')
    key('ctrl+a')
    key('BackSpace')
    focus('button', 'Check draft')
    key('space')
    wait_for(lambda: 'Title must contain 1–128 characters.' in text(find('dialog', 'Operation result')), 'Rust validation error is readable in the result dialog')
    wait_for(lambda: find('button', 'Close result').getState().contains(pyatspi.STATE_FOCUSED), 'error result receives focus')
    key('Escape')
    wait_for(lambda: find('button', 'Check draft').getState().contains(pyatspi.STATE_FOCUSED), 'error dismissal returns focus')
    # Re-enter the field using actual Tab traversal. A programmatic AT-SPI
    # focus request alone can leave Orca's Wayland browse cursor elsewhere.
    for _ in range(5):
        key('shift+Tab')
    wait_for(lambda: find('entry', 'Title').getState().contains(pyatspi.STATE_FOCUSED), 'keyboard returns from result to title editing')
    command('xdotool', 'type', '--clearmodifiers', '--delay', '60', 'Quantum physics')
    wait_for(lambda: text(find('entry', 'Title')) == 'Quantum physics', 'editing after an error preserves native keyboard routing')

    focus('button', 'Read preview')
    key('space')
    wait_for(lambda: find('button', 'Close preview').getState().contains(pyatspi.STATE_FOCUSED), 'modal initial focus')
    preview_nodes = nodes(find('dialog', 'Reading preview'))
    names = [{'role': node.getRoleName(), 'name': node.name} for node in preview_nodes]
    Path(f'/evidence/preview-{mode}.json').write_text(json.dumps(names, ensure_ascii=False, indent=2))
    if not any(node.getRoleName() == 'math' for node in preview_nodes):
        raise AssertionError('MathML has no accessible math role')
    results.append({'check': 'native MathML accessible role plus visible source', 'passed': True})
    read_from_focus()
    key('Tab')
    wait_for(lambda: find('button', 'Close preview').getState().contains(pyatspi.STATE_FOCUSED), 'native modal focus containment')
    key('Escape')
    wait_for(lambda: find('button', 'Read preview').getState().contains(pyatspi.STATE_FOCUSED), 'Escape restores preview trigger focus')
    focus('entry', 'Formula source (LaTeX)')
    wait_for(lambda: 'hbar' in text(find('entry', 'Formula source (LaTeX)')), 'formula source is accessible as editable text')
    focus('entry', 'Code source (never executed)')
    wait_for(lambda: 'return x * 2' in text(find('entry', 'Code source (never executed)')), 'code content is accessible without execution')

    focus('button', 'Choose a source file')
    key('space')
    wait_for(lambda: any('Choose a source' in node.name and node.getRoleName() in ('dialog', 'file chooser') for node in nodes()), 'native file picker opens')
    key('Escape')
    wait_for(lambda: 'No file selected' in text(feedback()), 'native file picker cancellation')
    wait_for(lambda: find('button', 'Choose a source file').getState().contains(pyatspi.STATE_FOCUSED), 'native picker restores focus')
    read_result('No file selected. Your edits are unchanged.')
    source = Path('/home/learner/memoquill-example.md')
    source.write_text('# Passive qualification source\nOriginal fixture; not an import.\n')
    focus('button', 'Choose a source file')
    key('space')
    wait_for(lambda: any('Choose a source' in node.name and node.getRoleName() in ('dialog', 'file chooser') for node in nodes()), 'native picker reopens')
    key('ctrl+l')
    command('xdotool', 'type', '--clearmodifiers', str(source))
    key('Return')  # Resolve the location before attempting the Open action.
    wait_for(lambda: selected_source() or open_enabled(), 'native file location validates')
    if not selected_source():
        native_open = next(node for node in nodes() if node.name == 'Open' and node.getRoleName() in ('button', 'push button'))
        if mode == 'wayland':
            key('alt+o')  # Real GTK mnemonic; no activation-token-dependent focus request.
        else:
            if not native_open.queryComponent().grabFocus():
                raise AssertionError('Cannot focus enabled native Open')
            wait_for(lambda: native_open.getState().contains(pyatspi.STATE_FOCUSED), 'enabled native Open receives focus')
            key('space')
    wait_for(selected_source, 'native file selection returns the chosen name')
    wait_for(lambda: find('button', 'Choose a source file').getState().contains(pyatspi.STATE_FOCUSED), 'selection restores picker focus')
    read_result('Selected memoquill-example.md. Contents were not read or executed.')

    focus('combo box', 'Appearance')
    key('alt+Down'); key('Home'); key('Down'); key('Down'); key('Return'); key('Escape')
    wait_for(lambda: any(node.name == 'Dark' and node.getState().contains(pyatspi.STATE_SELECTED) for node in nodes(find('combo box', 'Appearance'))), 'dark theme selected through native keyboard')
    focus('combo box', 'Text size')
    key('alt+Down'); key('Home'); key('Down'); key('Down'); key('Down'); key('Return'); key('Escape')
    wait_for(lambda: any(node.name == '200%' and node.getState().contains(pyatspi.STATE_SELECTED) for node in nodes(find('combo box', 'Text size'))), '200% text selected through native keyboard')
    capture('appearance')

    # An actual IBus engine commits Chinese text through the GTK IM context.
    edit_field('Title')
    results.append({'activeImeEngine': select_engine('libpinyin')})
    key('ctrl+a')
    command('xdotool', 'type', '--clearmodifiers', '--delay', '100', 'nihao')
    key('space')
    wait_for(lambda: any('\u4e00' <= char <= '\u9fff' for char in text(find('entry', 'Title'))), 'real IBus libpinyin composition commits Unicode')
    results.append({'imeCommittedText': text(find('entry', 'Title'))})
    edit_field('Learning notes')
    key('ctrl+a'); key('BackSpace')
    wait_for(lambda: not text(find('entry', 'Learning notes')).strip(), 'native editor selection and deletion')
    command('xdotool', 'type', '--clearmodifiers', '--delay', '100', 'nihao')
    key('space')
    wait_for(lambda: any('\u4e00' <= char <= '\u9fff' for char in text(find('entry', 'Learning notes'))), 'IBus composition commits in ProseMirror contenteditable')
    results.append({'editorImeCommittedText': text(find('entry', 'Learning notes'))})
    select_engine('xkb:us::eng')

    capture('screenshot')
    if os.environ.get('MEMOQUILL_STORAGE_AUDIT') == '1':
        storage_restart()
    time.sleep(2)
finally:
    Path(f'/evidence/journey-{mode}.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print(json.dumps({'mode': mode, 'checksPassed': sum(row.get('passed', False) for row in results), 'observations': len(results)}))
