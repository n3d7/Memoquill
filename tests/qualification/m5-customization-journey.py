"""Synthetic M5 subject and keyboard customization journey, only through owned X11/AT-SPI controls.

Run through verify.py native --scenario customization. No screenshots, direct IPC,
SQL edits, host desktop access or user learning data.
"""
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import time
import traceback

os.environ['MEMOQUILL_CAPTURE_SCREENSHOTS'] = '0'
spec = importlib.util.spec_from_file_location('driver', Path(__file__).with_name('atspi-journey.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)
d.capture = lambda *_: None
out = Path('/evidence')
results = d.results = []
original_nodes = d.nodes
pid = None
children = []


def root():
    app = next((a for a in d.pyatspi.Registry.getDesktop(0) if a.get_process_id() == pid), None)
    if app is None:
        raise LookupError('Owned app registration pending')
    return app


d.nodes = lambda node=None: original_nodes(root() if node is None else node)


def shown(value):
    return any(value in (d.text(n) + ' ' + (n.name or '')) for n in d.nodes())


def check(test, label):
    d.wait_for(test, label)
    (out / 'm5-settings-progress.json').write_text(json.dumps(results, indent=2))


def focus(name, roles=None):
    for node in d.nodes():
        if node.name == name and (roles is None or node.getRoleName() in roles):
            try:
                if node.queryComponent().grabFocus():
                    return node
            except NotImplementedError:
                pass
    raise LookupError('Focus target missing: ' + name)


def click(name):
    d.wait_for(lambda: focus(name).getState().contains(d.pyatspi.STATE_ENABLED), name + ' enabled', record=False)
    d.key('space')


def field(name, value):
    focus(name, ('entry', 'text', 'text area'))
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', '--delay', '1', value)
    d.key('Tab')


def scope(index):
    node = d.focus('combo box', 'Settings scope')
    box = node.queryComponent().getExtents(d.pyatspi.DESKTOP_COORDS)
    d.command('xdotool', 'mousemove', '--sync', str(box.x + box.width // 2), str(box.y + box.height // 2))
    d.command('xdotool', 'click', '1')
    d.key('Home')
    for _ in range(index):
        d.key('Down')
    d.key('Return')


def open_settings():
    click('Goals, focus and inherited settings')
    d.wait_for(lambda: d.find('combo box', 'Settings scope').getState().contains(d.pyatspi.STATE_ENABLED), 'settings loaded', record=False)


def launch():
    global pid
    log = (out / ('app-' + str(len(children)) + '.log')).open('w')
    child = subprocess.Popen(['memoquill'], stdout=log, stderr=log)
    children.append(child)
    pid = child.pid
    d.wait_for(lambda: d.find('button', 'Create subject').getState().contains(d.pyatspi.STATE_ENABLED), 'owned installed app ready', record=False)
    return child


def close(child):
    windows = d.command('xdotool', 'search', '--onlyvisible', '--pid', str(child.pid), '--name', 'Memoquill').splitlines()
    d.command('xdotool', 'windowactivate', '--sync', windows[-1])
    d.key('alt+F4')
    child.wait(timeout=15)
    check(lambda: child.returncode == 0, 'owned app closed normally with exit zero')
    locks = list(Path(os.environ['XDG_DATA_HOME']).rglob('*.lock'))
    assert len(locks) >= 2, 'Expected catalogue and initialized course locks'
    for path in locks:
        with path.open('r+b') as handle:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
            fcntl.flock(handle, fcntl.LOCK_UN)
    results.append({'check': 'catalogue and course owner locks released', 'passed': True})



def choose(name, label):
    if name != 'Settings scope': raise AssertionError('Unsupported scope selector')
    labels = ['Workspace defaults', 'Course overrides', 'Subject overrides', 'Temporary session overrides']
    if label not in labels: raise AssertionError('Unsupported scope label')
    scope(labels.index(label))

def keyboard_ack():
    check(lambda: not any(n.name == 'Discard keyboard edits' for n in d.nodes()), 'keyboard preferences acknowledged')

failure = None
try:
    first = launch()
    field('New subject', 'First synthetic subject'); click('Create subject')
    open_settings()
    click('Override learning goal'); field('course learning goal', 'Inherited course goal'); click('Save settings layer')
    check(lambda: shown('Inherited course goal · course'), 'course goal saved before subject overrides')
    choose('Settings scope', 'Subject overrides')
    click('Override learning goal'); field('subject learning goal', 'First subject goal'); click('Save settings layer')
    check(lambda: shown('First subject goal · subject'), 'first subject override attributed correctly')
    field('New subject', 'Second synthetic subject'); click('Create subject')
    check(lambda: shown('Inherited course goal · course'), 'second subject inherits rather than copying first overrides')
    click('Override learning goal'); field('subject learning goal', 'Second subject goal'); click('Save settings layer')
    check(lambda: shown('Second subject goal · subject'), 'second subject stores its own override')
    click('First synthetic subject')
    click('Override current focus'); field('subject current focus', 'Retained first subject draft')
    click('Second synthetic subject')
    check(lambda: shown('draft still belongs to the original subject'), 'subject switch retains draft ownership')
    click('Save settings layer')
    check(lambda: shown('Second subject goal · subject'), 'saving first draft does not overwrite selected second subject')
    click('First synthetic subject')
    check(lambda: shown('Retained first subject draft · subject'), 'first subject receives its retained draft')
    field('Settings shortcut', 'Ctrl+Alt+Shift+S')
    field('Search shortcut', 'Ctrl+Alt+Shift+S')
    click('Save keyboard preferences')
    check(lambda: shown('Use distinct'), 'duplicate keyboard bindings rejected without activation')
    field('Search shortcut', 'Ctrl+Alt+Shift+F')
    field('Workspace shortcut', 'Ctrl+Alt+Shift+M')
    click('Save keyboard preferences'); keyboard_ack()
    click('Goals, focus and inherited settings')
    focus('New subject', ('entry', 'text')); d.key('ctrl+alt+shift+s')
    check(lambda: not shown('Course keyboard bindings'), 'shortcuts do not intercept editable fields')
    focus('First synthetic subject'); d.key('ctrl+alt+shift+s')
    check(lambda: shown('Course keyboard bindings'), 'configured shortcut opens accessible Settings')
    focus('First synthetic subject'); d.key('ctrl+alt+shift+f')
    check(lambda: d.find('entry', 'Search titles, tags and text').getState().contains(d.pyatspi.STATE_FOCUSED), 'search shortcut returns focus to the real search field')
    close(first)
    second = launch()
    focus('First synthetic subject'); d.key('ctrl+alt+shift+s')
    check(lambda: shown('Course keyboard bindings'), 'keyboard bindings survive actual restart')
    click('First synthetic subject')
    check(lambda: shown('First subject goal · subject') and shown('Retained first subject draft · subject'), 'subject goal and focus survive actual restart')
    choose('Settings scope', 'Subject overrides')
    click('Reset this settings layer')
    check(lambda: shown('First subject goal · subject'), 'subject reset remains staged until Save')
    click('Save settings layer')
    check(lambda: shown('Inherited course goal · course'), 'subject reset restores inherited course goal')
    click('Second synthetic subject')
    check(lambda: shown('Second subject goal · subject'), 'reset keeps the other subject override')
    click('Export keyboard JSON')
    check(lambda: shown('"contractVersion": 1') and shown('Ctrl+Alt+Shift+S'), 'keyboard export retains versioned action bindings')
    click('Reset keyboard preferences')
    check(lambda: shown('Keyboard reset prepared'), 'keyboard reset is explicit and staged')
    click('Save keyboard preferences'); keyboard_ack()
    click('Goals, focus and inherited settings')
    focus('Second synthetic subject'); d.key('ctrl+alt+shift+s')
    check(lambda: not shown('Course keyboard bindings'), 'saved keyboard reset disables the binding')
    click('Goals, focus and inherited settings')
    check(lambda: shown('Course keyboard bindings') and shown('Second subject goal · subject'), 'Settings button remains accessible after keyboard reset without losing subject data')
    close(second)
except Exception:
    failure = traceback.format_exc(); (out / 'failure.txt').write_text(failure); raise
finally:
    for child in children:
        if child.poll() is None:
            child.terminate()
            try: child.wait(timeout=8)
            except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=5)
    (out / 'm3-journey.json').write_text(json.dumps({'scenario':'customization','checks':results,'failure':failure,'screenshots':False},indent=2))
