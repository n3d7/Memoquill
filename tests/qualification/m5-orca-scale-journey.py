"""Read the installed large catalog with Orca on the private X11 desktop."""
import json
import shutil
import subprocess
import time
import traceback
from pathlib import Path

base_path = Path('/input/m5-archive-search-journey.py')
definitions = base_path.read_text().split('failure = None\ntry:')[0]
exec(compile(definitions, str(base_path), 'exec'), globals())

metrics['orca'] = 'speech requests from private Orca instance'
metrics['audiblePlayback'] = 'not_run'
speech_log = out / 'orca-x11.log'
orca = None
failure = None
try:
    if shutil.which('orca') is None:
        raise RuntimeError('Orca is unavailable in the installed image')
    with (out / 'orca-x11-stderr.log').open('w') as stderr:
        orca = subprocess.Popen(
            ['orca', '--replace', '--debug', f'--debug-file={speech_log}'],
            stdout=stderr, stderr=stderr,
        )
    child = launch_scale('reopenUsefulViewSeconds', 1)
    assert orca.poll() is None, 'Orca must be running in the private desktop'
    metrics['initialStudyReadSeconds'] = eventually(
        lambda: shown('No unfinished saved session on this page.'),
        'initial study reads complete before opening a material', seconds=90)
    # Trigger actual focus and live-region changes after the AT-SPI startup.
    first = 'Scale resource 00000'
    focus(first, ('button', 'push button'))
    time.sleep(1.2)
    click(first)
    eventually(lambda: shown('fn square'), 'first material opens from keyboard focus', seconds=35)
    search_to_target(5000)
    time.sleep(1.2)
    search_to_target(9999)
    time.sleep(1.2)
    click('Show all materials')
    click('Last materials')
    check(lambda: shown('10000 materials. Showing 9981–10000.'),
          'Orca journey reaches the last material page')
    time.sleep(1.2)
    close(child)
except Exception as error:
    failure = str(error)
    (out / 'scale-traceback.log').write_text(traceback.format_exc())
finally:
    if orca and orca.poll() is None:
        orca.terminate()
        try:
            orca.wait(timeout=15)
        except subprocess.TimeoutExpired:
            orca.kill()
            orca.wait()
    if not failure:
        if not speech_log.exists():
            failure = 'Orca produced no debug evidence'
        else:
            with speech_log.open(errors='replace') as log:
                speech = '\n'.join(line for line in log if 'SPEECH OUTPUT:' in line)
            for label, phrase in (
                ('first material spoken', 'Scale resource 00000'),
                ('middle material spoken', 'Scale resource 05000'),
                ('last material spoken', 'Scale resource 09999'),
                ('catalog count spoken', '10000 materials'),
                ('search announced position', 'Matching material positions'),
            ):
                passed = phrase.casefold() in speech.casefold()
                results.append({'check': label, 'passed': passed})
            if any(not row.get('passed') for row in results):
                failure = 'Orca speech evidence is incomplete'
    (out / 'scale-metrics.json').write_text(json.dumps(metrics, indent=2))
    (out / 'm3-journey.json').write_text(json.dumps(
        {'scenario': 'scale-orca', 'checks': results, 'failure': failure}, indent=2))
    for child in children:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
if failure:
    raise SystemExit(failure)
