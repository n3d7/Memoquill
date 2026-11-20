"""Synthetic scale journey; X11/AT-SPI only, no screenshots/raw IPC/SQL."""
import json
import os
from pathlib import Path
import subprocess
import time
import traceback

base = Path('/input/m5-settings-journey.py').read_text().split('failure = None\ntry:')[0]
exec(compile(base, '/input/m5-settings-journey.py', 'exec'), globals())
def inspect_nodes(node=None):
    # Skip the options of collapsed selectors. The page can contain thousands
    # of selectable resources and lessons before the archive controls.
    pending = [root() if node is None else node]
    visited = []
    while pending and len(visited) < 5000:
        current = pending.pop()
        try:
            visited.append(current)
            if current.getRoleName() not in ('combo box', 'menu'):
                pending.extend(reversed(list(current)))
        except d.GLib.Error:
            pass
    return visited


d.nodes = inspect_nodes

def shown(value):
    return any(value in d.text(n) for n in d.nodes() if n.getRoleName() in ('statusbar','status bar','status','paragraph','text','entry','code','label','static','heading'))

def browser():
    return next(n for n in d.nodes() if n.name == 'Material browser')

def target_present(target):
    return any(n.name == target and n.getRoleName() in ('button','push button') and n.getState().contains(d.pyatspi.STATE_ENABLED) for n in d.nodes(browser()))

metrics = {'cacheState': 'OS cache state uncontrolled; reopen of the migrated workspace in private Podman', 'orca': 'not_run', 'uiBlocking': 'not_run', 'sorting': 'not_run in this journey'}

def eventually(test, label, seconds=90):
    start = time.monotonic()
    while time.monotonic() - start < seconds:
        try:
            if test():
                results.append({'check':label,'passed':True})
                return time.monotonic() - start
        except Exception:
            pass
        time.sleep(.2)
    raise AssertionError(label + ' did not become available')

def launch_scale(label, index):
    global pid
    log = (out / f'scale-app-{index}.log').open('w')
    with log:
        child = subprocess.Popen(['memoquill'], stdout=log, stderr=log)
    children.append(child)
    pid = child.pid
    metrics[label] = eventually(
        lambda: target_present('Scale resource 00000'),
        label + ' reaches the useful view', seconds=240)
    return child


def close_scale(child):
    click('Close current lesson')
    windows = d.command('xdotool', 'search', '--onlyvisible', '--pid', str(child.pid),
                        '--name', 'Memoquill').splitlines()
    d.command('xdotool', 'windowactivate', '--sync', windows[-1])
    start = time.monotonic()
    d.key('alt+F4')
    child.wait(timeout=45)
    metrics['closeSeconds'] = time.monotonic() - start
    assert child.returncode == 0, f'Application exited with {child.returncode}'
    results.append({'check': 'installed app exits normally after closing the lesson', 'passed': True})
    locks = list(Path(os.environ['XDG_DATA_HOME']).rglob('*.lock'))
    assert len(locks) >= 2
    for path in locks:
        with path.open('r+b') as handle:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
            fcntl.flock(handle, fcntl.LOCK_UN)
    results.append({'check': 'catalogue and workspace owner locks released', 'passed': True})


def select_last_lesson():
    label = 'Filter existing learning unit'
    field(label, 'Scale lesson 09999')
    check(lambda: shown('Showing 1–1 of 1 choices.'), 'lesson filter reaches the final lesson')
    focus('Existing learning unit', ('combo box',))
    d.key('space')
    d.key('Home')
    d.key('Down')
    d.key('Return')
    click('Open lesson')
    check(lambda: d.find('page tab', 'Scale lesson 09999').getState().contains(d.pyatspi.STATE_SELECTED),
          'keyboard opens the selected lesson beyond the first page')


failure = None
try:
    child = launch_scale('reopenUsefulViewSeconds', 1)
    check(lambda: shown('10000 materials. Showing 1–20.'), 'reopen preserves the catalog count')
    select_last_lesson()
    close_scale(child)
    metrics['normalExits'] = [process.returncode for process in children]
except Exception as error:
    failure = str(error)
    (out / 'scale-traceback.log').write_text(traceback.format_exc())
    diagnostics = {'childReturnCodeBeforeCleanup':children[-1].poll() if children else None,'nodes':[]}
    try:
        from itertools import islice
        for n in islice(d.nodes(), 500):
            diagnostics['nodes'].append({'role':n.getRoleName(),'name':(n.name or '')[:180],'attributes':n.getAttributes()})
    except Exception as diagnostic_error:
        diagnostics['treeError'] = str(diagnostic_error)
    try: diagnostics['memoryEvents'] = Path('/sys/fs/cgroup/memory.events').read_text()
    except OSError: pass
    (out / 'scale-diagnostics.json').write_text(json.dumps(diagnostics,indent=2))
finally:
    (out / 'scale-metrics.json').write_text(json.dumps(metrics,indent=2))
    (out / 'm3-journey.json').write_text(json.dumps({'scenario':'scale','checks':results,'failure':failure},indent=2))
    for child in children:
        if child.poll() is None:
            child.terminate()
            try: child.wait(timeout=5)
            except subprocess.TimeoutExpired: child.kill(); child.wait()
if failure:
    raise SystemExit(failure)
