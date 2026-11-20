"""Synthetic scale journey; X11/AT-SPI only, no screenshots/raw IPC/SQL."""
import json
import os
from pathlib import Path
import subprocess
import time
import traceback

base = Path('/input/m5-settings-journey.py').read_text().split('failure = None\ntry:')[0]
exec(compile(base, '/input/m5-settings-journey.py', 'exec'), globals())
def shown(value):
    return any(value in d.text(n) for n in d.nodes() if n.getRoleName() in ('statusbar','status bar','status','paragraph','text','entry','code','label','static','heading'))

def browser():
    return next(n for n in d.nodes() if n.name == 'Material browser')

def target_present(target):
    return any(n.name == target and n.getRoleName() in ('button','push button') and n.getState().contains(d.pyatspi.STATE_ENABLED) for n in d.nodes(browser()))

metrics = {'cacheState': 'generation-warm OS caches; first GUI process in private container', 'orca': 'not_run', 'uiBlocking': 'not_run', 'sorting': 'not_run: no sort control currently exposed'}

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

def status_node():
    for n in d.nodes(browser()):
        try:
            if n.getRoleName() not in ('statusbar', 'status bar', 'status'): continue
            value = d.text(n)
            if n.getRoleName() in ('statusbar', 'status bar', 'status') and ('matches in this batch' in value or 'No matches in this batch' in value):
                return n
        except Exception:
            pass
    raise LookupError('Search status missing')

def search_to_target(query, target):
    field('Search titles, tags and text', query)
    click('Search materials')
    eventually(lambda: 'matches in this batch' in d.text(status_node()), 'search first batch available')
    button = d.find('button', 'Continue search')
    status = status_node()
    batches = 1
    started = time.monotonic()
    while not target_present(target):
        assert button.getState().contains(d.pyatspi.STATE_ENABLED), 'target must remain reachable'
        assert button.queryComponent().grabFocus()
        d.key('space')
        # The native command is serialized; a changed progress status or heading
        # focus and enabled Continue establishes each acknowledged UI update.
        eventually(lambda: button.getState().contains(d.pyatspi.STATE_ENABLED) or target_present(target), 'search continuation acknowledged', seconds=30)
        batches += 1
        assert batches <= 55, 'bounded full-fixture scan'
    metrics.setdefault('targetSearches',[]).append({'query':query,'target':target,'continuations':batches-1,'seconds':time.monotonic()-started,'announcement':d.text(status)})
    check(lambda: target_present(target), 'keyboard search reaches ' + target)
    click(target)
    check(lambda: shown('fn square'), 'selected canonical document readable at ' + target)

failure = None
try:
    started = time.monotonic()
    log = (out / 'scale-app.log').open('w')
    child = subprocess.Popen(['memoquill'],stdout=log,stderr=log)
    children.append(child)
    pid = child.pid
    metrics['launchUsefulViewSeconds'] = eventually(lambda: d.find('button','Create subject').getState().contains(d.pyatspi.STATE_ENABLED) and target_present('Scale resource 00000'), 'full 10000-material useful view', seconds=120)
    check(lambda: shown('10000 materials. Showing 1–20.'), 'full count and first-page position announced')
    check(lambda: target_present('Scale resource 00000'), 'beginning material visible')
    first = d.find('button','Scale resource 00000')
    check(lambda: bool(first.name) and first.getRoleName() in ('button','push button'), 'material exposes accessible name and role')
    click('Next materials')
    check(lambda: shown('10000 materials. Showing 21–40.'), 'keyboard reaches next material page with position and total')
    heading = d.find('heading','Available materials')
    check(lambda: heading.getState().contains(d.pyatspi.STATE_FOCUSED), 'page change returns focus to material heading')
    click('Previous materials')
    check(lambda: shown('10000 materials. Showing 1–20.'), 'keyboard returns to beginning page')
    search_to_target('resource 05000','Scale resource 05000')
    search_to_target('resource 09999','Scale resource 09999')
    click('Show all materials')
    check(lambda: shown('10000 materials. Showing 1–20.'), 'returning from search restores bounded browser')
    close(child)
    metrics['normalExit'] = child.returncode
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
