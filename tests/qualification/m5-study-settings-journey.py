"""Synthetic M5 canonical settings and positive Due journey, only through owned X11/AT-SPI controls.

Run through verify.py native --scenario study-settings. No screenshots, direct IPC,
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
    focus(name, ('entry', 'text', 'text area', 'spin button', 'spinbutton'))
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', '--delay', '1', value)
    d.key('Tab')


def temporary_scope():
    node = d.focus('combo box', 'Settings scope')
    box = node.queryComponent().getExtents(d.pyatspi.DESKTOP_COORDS)
    d.command('xdotool', 'mousemove', '--sync', str(box.x + box.width // 2), str(box.y + box.height // 2))
    d.command('xdotool', 'click', '1')
    d.key('End')
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



def choose(name, index):
    node = d.focus('combo box', name)
    box = node.queryComponent().getExtents(d.pyatspi.DESKTOP_COORDS)
    d.command('xdotool', 'mousemove', '--sync', str(box.x + box.width // 2), str(box.y + box.height // 2))
    d.command('xdotool', 'click', '1')
    d.key('Home')
    for _ in range(index): d.key('Down')
    d.key('Return')

def choose_label(name, label):
    node = d.focus('combo box', name)
    names = []
    for n in d.nodes(node):
        if n is not node and n.name and n.name != name and n.name not in names: names.append(n.name)
    if label not in names: raise AssertionError('Option missing: ' + repr((name, label, names)))
    choose(name, names.index(label))

def policy(zone):
    field('IANA time zone', zone)
    field('Study day starts at hour', '0')
    click('Save recall policy')
    d.wait_for(lambda: d.find('button', 'Save recall policy').getState().contains(d.pyatspi.STATE_ENABLED), 'policy acknowledged', record=False)

def study_shown(value):
    section = next(n for n in d.nodes() if n.name == 'Study lesson')
    return any(value in (d.text(n) + ' ' + (n.name or '')) for n in d.nodes(section))

failure = None
try:
    first = launch()
    open_settings()
    click('Override learning goal')
    field('course learning goal', 'Canonical synthetic goal')
    click('Save settings layer')
    check(lambda: shown('Canonical synthetic goal · course'), 'course settings acknowledged before Start')
    temporary_scope()
    click('Override current focus')
    field('session current focus', 'Frozen synthetic focus')
    click('Apply temporary settings')
    check(lambda: shown('Frozen synthetic focus · session'), 'temporary setting acknowledged before Start')
    # A synthetic policy-day change exercises Due without changing host/container clocks.
    policy('Etc/GMT+12')
    field('New lesson title', 'M5 canonical recall')
    click('Create lesson')
    click('Edit lesson')
    choose_label('Component type', 'Exercise')
    choose('New exercise family', 0)
    click('Add component')
    field('Exercise prompt', 'Synthetic recall {{answer}}')
    choose('Recall format', 1)
    field('Expected answer', 'Answer')
    click('Save exercise revision for this lesson')
    click('Save lesson')
    check(lambda: shown('Lesson saved locally.'), 'synthetic recall lesson published locally')
    field('New session item limit', '1')
    choose('New session activities', 1)
    click('Start practice')
    check(lambda: study_shown('Session goal: Canonical synthetic goal') and study_shown('Session focus: Frozen synthetic focus'), 'Start persists effective course and temporary settings')
    check(lambda: study_shown('Saved workload: 1 items maximum; activity family: recall'), 'Start records the explicit activity and item budget')
    field('Your response', 'Canonical acknowledged draft')
    choose('External help', 1)
    click('Save response draft')
    check(lambda: d.find('button', 'Submit saved response').getState().contains(d.pyatspi.STATE_ENABLED), 'canonical response draft acknowledged')
    field('New session item limit', '64')
    choose('New session activities', 0)
    check(lambda: study_shown('Saved workload: 1 items maximum; activity family: recall'), 'new-session controls do not rewrite an existing workload')
    click('Close current lesson')
    field('session current focus', 'Changed view focus')
    click('Apply temporary settings')
    check(lambda: shown('Changed view focus · session'), 'view defaults changed after Start')
    close(first)
    second = launch()
    click('Refresh start actions')
    click('Resume saved session')
    check(lambda: study_shown('Session focus: Frozen synthetic focus'), 'Resume restores frozen settings after actual restart and changed defaults')
    check(lambda: d.text(d.find('entry', 'Your response')) == 'Canonical acknowledged draft', 'Resume restores canonical response with its settings')
    check(lambda: study_shown('Session context: independent'), 'Resume restores the canonical phase')
    check(lambda: study_shown('Saved workload: 1 items maximum; activity family: recall'), 'Resume restores the pinned activity budget after restart')
    choose('Recall schedule rating', 1)
    click('Submit saved response')
    check(lambda: study_shown('Completed 1 of 1'), 'explicit Again rating submits a scheduled recall')
    click('Return to start actions')
    policy('Pacific/Kiritimati')
    click('Refresh start actions')
    check(lambda: d.find('button', 'Review due recall').getState().contains(d.pyatspi.STATE_ENABLED), 'positive scheduled Due offered after synthetic policy-day advance')
    click('Review due recall')
    check(lambda: study_shown('Session goal: Canonical synthetic goal') and study_shown('Session focus: Not set'), 'Due starts a new canonical session with current settings')
    field('Your response', 'Due acknowledged draft')
    choose('External help', 1)
    click('Save response draft')
    check(lambda: d.find('button', 'Submit saved response').getState().contains(d.pyatspi.STATE_ENABLED), 'Due response acknowledged')
    close(second)
    third = launch()
    click('Resume saved session')
    check(lambda: d.text(d.find('entry', 'Your response')) == 'Due acknowledged draft', 'latest Due session resumes after a second actual restart')
    check(lambda: study_shown('Session focus: Not set'), 'Due snapshot stays distinct from the prior session')
    close(third)
except Exception:
    failure = traceback.format_exc()
    (out / 'failure.txt').write_text(failure)
    raise
finally:
    for child in children:
        if child.poll() is None:
            child.terminate()
            try: child.wait(timeout=8)
            except subprocess.TimeoutExpired:
                child.kill(); child.wait(timeout=5)
    (out / 'm3-journey.json').write_text(json.dumps({'scenario': 'study-settings', 'checks': results, 'failure': failure, 'screenshots': False}, indent=2))
