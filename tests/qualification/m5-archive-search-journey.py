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

metrics = {'cacheState': 'OS cache state uncontrolled; reopen of the migrated workspace in private Podman', 'orca': 'not_run', 'uiBlocking': 'not_run', 'sorting': 'material archive title descending'}

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


def search_to_target(index):
    query = f'resource {index:05d}'
    target = f'Scale resource {index:05d}'
    field('Search titles, tags and text', query)
    start = time.monotonic()
    click('Search materials')
    eventually(lambda: target_present(target), 'one search action reaches ' + target, seconds=85)
    metrics.setdefault('targetSearches', []).append({
        'target': target, 'seconds': time.monotonic() - start
    })
    check(lambda: shown(f'Matching material positions {index+1}–{index+1} of 10000.'),
          'search announces the target position and total')
    click(target)
    check(lambda: shown('fn square'), 'selected canonical document at ' + target)
    check(lambda: 'current:page' in d.find('button', target).getAttributes(),
          'selected search result exposes current page')


def archive_target(target):
    click('Manage archived items')
    archive = d.find('landmark', 'Material archive controls')
    assert len([n for n in d.nodes(archive) if n.getRoleName() == 'button' and
                (n.name or '').startswith('Archive material ')]) == 20
    focus('Sort material archive items', ('combo box',))
    d.key('space')
    d.key('End')
    d.key('Return')
    check(lambda: d.find('button', 'Archive material ' + target) is not None,
          'descending order exposes the final material on the first archive page')
    field('Filter material archive items', target)
    check(lambda: shown('Filter matches: 1 of 10000 material items.'),
          'archive reports one filtered material out of the full catalog')
    start = time.monotonic()
    click('Archive material ' + target)
    eventually(lambda: shown('9999 materials.'), 'archiving updates the active count', seconds=75)
    metrics['archiveSeconds'] = time.monotonic() - start
    field('Search titles, tags and text', 'resource 09999')
    click('Search materials')
    eventually(lambda: shown('No matches in this batch. Search complete.'),
          'the archived result disappears from search', seconds=85)
    click('Show all materials')
    start = time.monotonic()
    click('Restore material ' + target)
    eventually(lambda: shown('10000 materials.'), 'restoring recovers the active count', seconds=75)
    metrics['restoreSeconds'] = time.monotonic() - start
    click('Manage archived items')


failure = None
try:
    child = launch_scale('reopenUsefulViewSeconds', 1)
    check(lambda: shown('10000 materials. Showing 1–20.'), 'reopen preserves the catalog count')
    archive_target('Scale resource 09999')
    search_to_target(9999)
    close(child)
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
