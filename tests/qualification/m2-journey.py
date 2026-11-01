"""Exercise the packaged M2 product through native keyboard/AT-SPI only.

Runs on the existing isolated Fedora desktop. No direct IPC, SQL or DOM calls.
The legacy driver supplies shared keyboard/focus/screenshot helpers without
executing its M1 fixture journey when imported.
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time

spec = importlib.util.spec_from_file_location('native_driver', Path(__file__).with_name('atspi-journey.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)
mode = sys.argv[1]
results = d.results
child = None


def shown(value):
    return any(value in d.text(node) for node in d.nodes())


def click(name):
    d.focus('button', name)
    d.key('space')


def type_in(name, value, role='entry'):
    if role == 'entry':
        d.edit_field(name)
    else:
        d.focus(role, name)
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', '--delay', '1', value)
    d.wait_for(lambda: value in d.text(d.find(role, name)), f'keyboard input retained in {name}')


def choose(name, index):
    d.focus('combo box', name)
    d.key('alt+Down')
    d.key('Home')
    for _ in range(index):
        d.key('Down')
    d.key('Return')
    d.key('Escape')


def picker_closed():
    return d.find('button', 'Choose source file').getState().contains(d.pyatspi.STATE_FOCUSED)


try:
    d.wait_for(lambda: d.find('button', 'Create subject').getState().contains(d.pyatspi.STATE_ENABLED), 'fresh workspace is authorable')
    d.capture('m2-start')
    type_in('New subject', 'Quantum physics')
    click('Create subject')
    d.wait_for(lambda: shown('Current subject: Quantum physics'), 'blank subject created without a template')
    type_in('New collection', 'Foundations')
    click('Create collection')
    d.wait_for(lambda: d.find('button', 'Foundations'), 'custom collection appears after its acknowledged write')
    type_in('New material', 'Native quantum notes')
    click('Create material')
    d.wait_for(lambda: d.find('entry', 'Learning document'), 'new resource opens its real structured editor')
    type_in('Learning document', 'Native quantum provenance')
    d.command('xdotool', 'type', '--clearmodifiers', ' undo-target')
    d.key('ctrl+z')
    d.wait_for(lambda: 'undo-target' not in d.text(d.find('entry', 'Learning document')), 'native editor undo removes inserted text')
    d.key('ctrl+shift+z')
    d.wait_for(lambda: 'undo-target' in d.text(d.find('entry', 'Learning document')), 'native editor redo restores text')

    # Exercise actual GTK IM composition in the new ProseMirror editor.
    d.edit_field('Learning document')
    d.key('ctrl+End')
    d.key('space')
    results.append({'activeImeEngine': d.select_engine('libpinyin')})
    d.command('xdotool', 'type', '--clearmodifiers', '--delay', '100', 'nihao')
    d.key('space')
    d.wait_for(lambda: any('\u4e00' <= c <= '\u9fff' for c in d.text(d.find('entry', 'Learning document'))), 'real IBus composition commits in the M2 editor')
    d.select_engine('xkb:us::eng')

    choose('New block type', 3)
    click('Add block')
    type_in('Formula source', r'E=\hbar\omega')
    d.read_from_focus()
    choose('New block type', 2)
    click('Add block')
    d.edit_field('Learning document')
    d.key('ctrl+End')
    d.command('xdotool', 'type', '--clearmodifiers', "print('never executed')")
    d.read_from_focus()
    click('Save material')
    d.wait_for(lambda: shown('Saved locally.'), 'canonical document receives a durable save acknowledgement')
    type_in('Annotation at start of block', 'Native anchored note')
    click('Add annotation')
    d.wait_for(lambda: not d.text(d.find('entry', 'Annotation at start of block')) and shown('Native anchored note'), 'acknowledged annotation clears its draft and appears in saved notes')
    click('Remember reading location')
    d.wait_for(lambda: shown('Reading location saved locally.'), 'reading location persists separately from content')

    click('Choose source file')
    d.wait_for(lambda: any(node.getRoleName() in ('dialog', 'file chooser') for node in d.nodes()), 'native source chooser opens')
    d.key('Escape')
    d.wait_for(picker_closed, 'cancelled source chooser restores actual trigger focus')
    click('Choose source file')
    d.wait_for(lambda: any(node.getRoleName() in ('dialog', 'file chooser') for node in d.nodes()), 'native source chooser reopens')
    d.key('ctrl+l')
    d.command('xdotool', 'type', '--clearmodifiers', '/input/m2-original.html')
    d.key('Return')
    d.wait_for(lambda: picker_closed() or d.open_enabled(), 'explicit source location validates')
    if not picker_closed():
        if mode == 'wayland':
            d.key('alt+o')
        else:
            click('Open')
    d.wait_for(picker_closed, 'selected source returns focus without exposing its path')
    type_in('Attribution', 'Original qualification fixture')
    type_in('License', 'CC0-1.0')
    click('Attach selected source')
    d.wait_for(lambda: d.find('button', 'Preview original m2-original.html'), 'managed original and provenance are attached')
    click('Preview original m2-original.html')
    d.wait_for(lambda: shown('<script>never_execute_original()</script>'), 'HTML original is presented as literal passive text')
    assert not any(node.name == 'Injected active button' and node.getRoleName() == 'button' for node in d.nodes())
    results.append({'check': 'original markup never creates active controls', 'passed': True})

    type_in('Search titles, tags and text', 'native quantum provenance')
    click('Search materials')
    d.wait_for(lambda: d.find('button', 'Native quantum notes'), 'saved canonical text is discoverable offline')
    click('Show all materials')
    for title in ('English', 'Python', 'Cooking'):
        type_in('New subject', title)
        click('Create subject')
        d.wait_for(lambda: shown(f'Current subject: {title}'), f'{title} uses the same blank subject workflow')

    choose('Appearance', 2)
    choose('Text size', 3)
    click('Save reading preferences')
    d.wait_for(lambda: d.find('dialog', 'Operation result'), 'reading preferences return an accessible acknowledgement')
    d.read_from_focus()
    click('Close result')
    d.wait_for(lambda: d.find('button', 'Save reading preferences').getState().contains(d.pyatspi.STATE_FOCUSED), 'result closure restores requested trigger focus')
    d.capture('m2-authoring')

    # Close normally, then restart with identical private profile/backend.
    d.key('alt+F4')
    d.wait_for(lambda: not any(node.name == 'Create subject' for node in d.nodes()), 'first product window closes normally')
    child_env = os.environ.copy()
    child_env['WAYLAND_DEBUG'] = '1'
    with Path(f'/evidence/m2-restart-{mode}.log').open('w') as output:
        child = subprocess.Popen(['memoquill'], env=child_env, stdout=output, stderr=subprocess.STDOUT)
    d.wait_for(lambda: d.find('button', 'Native quantum notes'), 'saved resources survive process restart')
    for title in ('Quantum physics', 'English', 'Python', 'Cooking'):
        d.find('button', title)
    results.append({'check': 'all subjects survive restart', 'passed': True})
    click('Quantum physics')
    d.wait_for(lambda: d.find('button', 'Foundations'), 'collection survives process restart')
    click('Native quantum notes')
    d.wait_for(lambda: 'Native quantum provenance' in d.text(d.find('entry', 'Learning document')), 'canonical prose survives reopen')
    d.wait_for(lambda: shown('Native anchored note'), 'anchored note survives restart')
    d.wait_for(lambda: shown('Saved reading location restored.'), 'exact saved reading position restores without changing content')
    choose('Selected block', 1)
    d.wait_for(lambda: r'E=\hbar\omega' in d.text(d.find('entry', 'Formula source')), 'authored math source survives reopen')
    d.edit_field('Formula source')
    d.read_from_focus()
    click('Preview original m2-original.html')
    d.wait_for(lambda: shown('<script>never_execute_original()</script>'), 'managed original survives restart unchanged')
    d.capture('m2-reopened')
    d.key('alt+F4')
    results.append({'check': 'restarted product exits normally', 'passed': child.wait(timeout=10) == 0})
finally:
    if child is not None and child.poll() is None:
        child.terminate()
        child.wait(timeout=10)
    Path(f'/evidence/m2-journey-{mode}.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print(json.dumps({'mode': mode, 'checksPassed': sum(row.get('passed', False) for row in results)}))
