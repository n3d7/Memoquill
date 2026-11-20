"""Synthetic M5 settings journey, only through owned X11/AT-SPI controls.

Run through verify.py native --scenario settings. No screenshots, direct IPC,
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


failure = None
try:
    first = launch()
    open_settings()
    click('Override learning goal')
    field('course learning goal', 'Synthetic course goal')
    click('Save settings layer')
    check(lambda: shown('Synthetic course goal · course'), 'course goal saved and attributed')
    scope(0)
    click('Override learning goal')
    field('workspace learning goal', 'Synthetic workspace goal')
    click('Override current focus')
    field('workspace current focus', 'Synthetic workspace focus')
    click('Save settings layer')
    check(lambda: shown('Synthetic workspace focus · workspace'), 'workspace focus inherited')
    check(lambda: shown('Synthetic course goal · course'), 'workspace change retains course override')
    scope(2)
    click('Override current focus')
    field('session current focus', 'Temporary synthetic focus')
    click('Apply temporary settings')
    check(lambda: shown('Temporary synthetic focus · session'), 'temporary session override applies')
    click('Reset this settings layer')
    check(lambda: shown('Temporary synthetic focus · session'), 'reset is staged before explicit apply')
    click('Apply temporary settings')
    check(lambda: shown('Synthetic workspace focus · workspace'), 'session reset restores inherited focus')
    click('Override current focus')
    field('session current focus', 'Discard on restart')
    click('Apply temporary settings')
    check(lambda: shown('Discard on restart · session'), 'temporary override ready for restart check')
    close(first)
    second = launch()
    open_settings()
    check(lambda: shown('Synthetic course goal · course') and shown('Synthetic workspace focus · workspace'), 'saved course and workspace settings survive actual restart')
    check(lambda: not shown('Discard on restart'), 'temporary override does not survive restart')
    field('New course', 'Synthetic second course')
    click('Create course')
    d.wait_for(lambda: d.find('button', 'Create subject').getState().contains(d.pyatspi.STATE_ENABLED), 'new course ready', record=False)
    open_settings()
    check(lambda: shown('Synthetic workspace goal · workspace') and not shown('Synthetic course goal · course'), 'new course inherits workspace without another course goal')
    close(second)
except Exception:
    failure = traceback.format_exc()
    (out / 'failure.txt').write_text(failure)
    raise
finally:
    for child in children:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=8)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=5)
    (out / 'm3-journey.json').write_text(json.dumps({'scenario': 'settings', 'checks': results, 'failure': failure, 'screenshots': False}, indent=2))
