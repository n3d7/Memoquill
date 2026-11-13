"""Native M3 ingestion journey; later package/recovery checks are separately recorded.

Runs only in the isolated qualification desktop. No direct IPC/SQL/DOM mutation.
"""
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import zipfile

os.environ.setdefault('MEMOQUILL_CAPTURE_SCREENSHOTS', os.environ.get('MEMOQUILL_M3_SCREENSHOTS', '0'))
spec = importlib.util.spec_from_file_location('native_driver', Path(__file__).with_name('atspi-journey.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


class Journal(list):
    def append(self, result):
        super().append(result)
        Path('/evidence/m3-progress.json').write_text(json.dumps(self, ensure_ascii=False, indent=2))


results = d.results = Journal()


def shown(value):
    return any(value in (d.text(node) + ' ' + (node.name or '')) for node in d.nodes())


def click(name):
    d.wait_for(lambda: d.find('button', name).getState().contains(d.pyatspi.STATE_ENABLED), f'{name} enabled', record=False)
    node = d.focus('button', name)
    d.wait_for(lambda: node.getState().contains(d.pyatspi.STATE_SHOWING), f'{name} in viewport', record=False)
    # Keyboard activation targets the verified focused button even while the long
    # page scrolls; screen coordinates can become stale after focus.
    d.key('space')


def field(name, value):
    for node in d.nodes():
        if node.name == name and node.getRoleName() in ('entry', 'spin button', 'spinbutton'):
            if node.queryComponent().grabFocus():
                d.key('ctrl+a')
                d.command('xdotool', 'type', '--clearmodifiers', '--delay', '1', value)
                d.key('Tab')
                return
    raise LookupError(f'Editable field not found: {name}')


def attach(path):
    filename = Path(path).name
    click('Choose source file')
    d.wait_for(lambda: any(node.getRoleName() in ('dialog', 'file chooser') for node in d.nodes()), 'source chooser opened')
    d.key('ctrl+l')
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', path)
    d.key('Return')
    d.wait_for(lambda: shown('Selected:') or d.open_enabled(), 'source path validated')
    if not shown('Selected:'):
        click('Open')
    d.wait_for(lambda: shown('Selected:'), 'source selected through native chooser')
    field('Attribution', 'Original synthetic M3 qualification fixture')
    field('License', 'Original project test fixture')
    click('Attach selected source')
    d.wait_for(lambda: d.find('button', f'Preview original {filename}'), f'{filename} attached with provenance')


def save_original(filename):
    destination = Path('/evidence/originals') / filename
    destination.parent.mkdir(exist_ok=True)
    click(f'Save original {filename}')
    d.wait_for(lambda: any(node.getRoleName() in ('dialog', 'file chooser') for node in d.nodes()), f'{filename} native destination chooser opened')
    d.key('ctrl+l')
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', str(destination))
    d.key('Return')
    d.wait_for(lambda: published_path(destination) or visible_button('Save'), 'native save destination accepted', record=False)
    if not published_path(destination) and visible_button('Save'):
        try:
            click('Save')
        except LookupError:
            pass  # Native save can finish between the predicate and focus; bytes below remain mandatory.
    d.wait_for(lambda: published_path(destination), f'{filename} original copied')
    destination = published_path(destination)
    expected = hashlib.sha256((Path('/input') / filename).read_bytes()).hexdigest()
    actual = hashlib.sha256(destination.read_bytes()).hexdigest()
    if actual != expected:
        raise AssertionError(f'Original bytes changed: {filename}')
    if destination.name != filename:
        raise AssertionError(f'Native save filename mismatch: expected {filename}, got {destination.name}')
    results.append({'check': f'preserved original copy {filename}', 'passed': True, 'sha256': actual, 'actual_basename': destination.name})



def published_path(path):
    path = Path(path)
    # GTK may append the suggested extension to a typed absolute name.
    for candidate in (path, path.with_name(path.name + path.suffix)):
        if candidate.is_file():
            return candidate
    return None


def visible_button(name):
    try:
        state = d.find('button', name).getState()
        return state.contains(d.pyatspi.STATE_ENABLED) and state.contains(d.pyatspi.STATE_SHOWING)
    except LookupError:
        return False


def chooser_visible():
    return any(node.getRoleName() in ('dialog', 'file chooser') and node.getState().contains(d.pyatspi.STATE_SHOWING) for node in d.nodes())


def choose_native_path(path, saving=False):
    d.wait_for(lambda: any(node.getRoleName() in ('dialog', 'file chooser') for node in d.nodes()), 'package native chooser opened')
    d.key('ctrl+l')
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', str(path))
    d.key('Return')
    if saving:
        d.wait_for(lambda: published_path(path) or visible_button('Save'), 'native package destination accepted', record=False)
        if not published_path(path) and visible_button('Save'):
            try:
                click('Save')
            except LookupError:
                pass  # Publication may have completed while the chooser closed.
        d.wait_for(lambda: published_path(path), 'package published to selected destination')
        return published_path(path)
    else:
        d.wait_for(lambda: not chooser_visible() or visible_button('Open'), 'native package source accepted', record=False)
        if visible_button('Open'):
            try:
                click('Open')
            except LookupError:
                pass  # Inspection may already have dismissed the chooser.


def package_roundtrips():
    packages = (
        ('Export full recovery package', 'native', 'M3 native restored course'),
        ('Export shareable content package', 'portable', 'M3 portable imported course'),
    )
    exported = {}
    for label, kind, _ in packages:
        destination = Path('/evidence') / f'{kind}.zip'
        click(label)
        destination = choose_native_path(destination, saving=True)
        exported[kind] = destination
        with zipfile.ZipFile(destination) as archive:
            names = set(archive.namelist())
            manifest = json.loads(archive.read('manifest.json'))
            if kind == 'native':
                assert 'database.sqlite3' in names and manifest['complete'] is True
                assert any(name.startswith('assets/') for name in names)
            else:
                assert 'database.sqlite3' not in names
                assert 'content.ndjson' in names and 'readable/content.txt' in names
                assert manifest['kind'] == 'portableContent'
                assert manifest['originalsRequested'] is False
                assert manifest['intentionallyOmittedAssets']
                assert not any(name.startswith('assets/') for name in names)
        results.append({'check': f'{kind} native-selected export payload and completeness', 'passed': True, 'sha256': hashlib.sha256(destination.read_bytes()).hexdigest()})
    for _, kind, title in packages:
        click('Choose package to inspect')
        choose_native_path(exported[kind])
        d.wait_for(lambda: d.find('entry', 'New course title'), f'{kind} package inspection review available')
        field('New course title', title)
        click('Confirm import into new course')
        d.wait_for(lambda: d.find('button', title), f'{kind} new course admitted')
        click(title)
        d.wait_for(lambda: d.find('button', 'First quantum record'), f'{kind} canonical content restored')
        results.append({'check': f'{kind} inspected and confirmed into new isolated course', 'passed': True})


def select_index(name, index):
    node = d.focus('combo box', name)
    bounds = node.queryComponent().getExtents(d.pyatspi.DESKTOP_COORDS)
    d.command('xdotool', 'mousemove', '--sync', str(bounds.x + bounds.width // 2), str(bounds.y + bounds.height // 2))
    d.command('xdotool', 'click', '1')
    d.key('Home')
    for _ in range(index):
        d.key('Down')
    d.key('Return')


def reimport_and_undo():
    baseline = next(node.name for node in d.nodes() if node.getRoleName() in ('button', 'push button') and node.name.startswith('Open import '))
    click('First quantum record')
    field('Learning document', 'Human authored preserved')
    click('Save material')
    d.wait_for(lambda: not d.find('button', 'Discard unsaved changes').getState().contains(d.pyatspi.STATE_ENABLED), 'human edit saved before reimport')
    click('M3 original source holder')
    attach('/input/import-changed.csv')
    select_index('Saved source', 1)
    if os.environ.get('MEMOQUILL_M3_SCREENSHOTS') == '1':
        d.capture('m3-selected-changed-source')
    click('Create import preview')
    d.wait_for(lambda: shown('Changed source body λ'), 'changed source selected and staged')
    field('Title column', '2')
    field('Body column', '3')
    select_index('Reimport from', 1)
    d.wait_for(lambda: d.find('button', 'Compare reimport').getState().contains(d.pyatspi.STATE_ENABLED), 'reimport comparison ready')
    click('Compare reimport')
    d.wait_for(lambda: shown('alpha: conflict') and shown('Human authored preserved') and shown('Changed source body λ'), 'prior human and incoming versions displayed')
    if d.find('check box', 'Select record 1').getState().contains(d.pyatspi.STATE_ENABLED):
        raise AssertionError('Human conflict selectable without approval')
    if os.environ.get('MEMOQUILL_M3_SCREENSHOTS') == '1':
        d.capture('m3-human-conflict')
    approval = next(node.name for node in d.nodes() if node.getRoleName() == 'check box' and node.name.startswith('Explicitly replace current human edits for alpha'))
    d.focus('check box', approval)
    d.key('space')
    d.wait_for(lambda: d.find('check box', 'Select record 1').getState().contains(d.pyatspi.STATE_ENABLED), 'explicit conflict approval enables selection')
    d.focus('check box', 'Select record 1')
    d.key('space')
    click('Apply selected records')
    d.wait_for(lambda: shown('1/3 applied'), 'reviewed changed record applied')
    click('Preview undo')
    d.wait_for(lambda: d.find('button', 'Confirm undo'), 'compensating undo preview available')
    click('Confirm undo')
    d.wait_for(lambda: shown('undone'), 'reimport compensation acknowledged')
    click('First quantum record')
    d.wait_for(lambda: 'Human authored preserved' in d.text(d.find('entry', 'Learning document')), 'undo restores human-authored revision')
    for title in ('Second quantum record', 'Last quantum record'):
        d.find('button', title)
    results.append({'check': 'partial undo retains parent materials', 'passed': True})
    click('M3 original source holder')
    click(baseline)
    click('Preview undo')
    d.wait_for(lambda: shown('Undo is unavailable because content or dependencies changed.'), 'initial import undo refuses later human edit')


def json_property_import(source_index=2):
    attach('/input/import.json')
    select_index('Saved source', source_index)
    click('Create import preview')
    d.wait_for(lambda: d.find('combo box', 'Key property'), 'versioned JSON property mapping available')
    select_index('Key property', 2)
    select_index('Title property', 3)
    select_index('Body property', 1)
    if d.find('check box', 'Select record 3').getState().contains(d.pyatspi.STATE_ENABLED):
        raise AssertionError('Ambiguous duplicate-property record is applyable')
    click('Exclude record 3')
    d.wait_for(lambda: d.find('button', 'Reopen record 3'), 'ambiguous JSON outcome recorded explicitly')
    for index in (1, 2):
        node = d.focus('check box', f'Select record {index}')
        bounds = node.queryComponent().getExtents(d.pyatspi.DESKTOP_COORDS)
        d.command('xdotool', 'mousemove', '--sync', str(bounds.x + bounds.width // 2), str(bounds.y + bounds.height // 2))
        d.command('xdotool', 'click', '1')
        d.wait_for(lambda index=index: d.find('check box', f'Select record {index}').getState().contains(d.pyatspi.STATE_CHECKED), f'JSON record {index} checked')
    click('Apply selected records')
    d.wait_for(lambda: shown('2/3 applied'), 'named JSON selection applied')
    for title in ('JSON first material', 'JSON second material'):
        d.wait_for(lambda title=title: d.find('button', title), f'named JSON material created: {title}')
    d.wait_for(lambda: shown('0 pending.'), 'every JSON record reconciled')


def main():
    d.wait_for(lambda: d.find('button', 'Create subject').getState().contains(d.pyatspi.STATE_ENABLED), 'course workspace ready')
    window = d.command('xdotool', 'search', '--name', 'Memoquill').splitlines()[-1]
    d.command('xdotool', 'windowactivate', '--sync', window)
    field('New subject', 'M3 source qualification')
    click('Create subject')
    field('New collection', 'Imported research')
    click('Create collection')
    field('New material', 'M3 original source holder')
    click('Create material')
    d.wait_for(lambda: d.find('entry', 'Learning document'), 'source holder opens')
    scenario = os.environ.get('MEMOQUILL_M3_SCENARIO', 'full')
    if scenario in ('originals', 'save-dialog'):
        attach('/input/import.csv')
        if scenario == 'save-dialog':
            before = set(Path('/evidence').rglob('*'))
            click('Save original import.csv')
            d.wait_for(chooser_visible, 'cancel scenario chooser visible')
            click('Cancel')
            d.wait_for(lambda: not chooser_visible(), 'cancel closes chooser')
            after = set(Path('/evidence').rglob('*'))
            if any(p.is_file() for p in after - before):
                raise AssertionError('Cancelled save unexpectedly published a file')
            results.append({'check': 'native save cancellation publishes no file', 'passed': True})
        save_original('import.csv')
        d.key('alt+F4')
        return
    if scenario == 'json':
        os.environ['MEMOQUILL_M3_JSON_ONLY'] = '1'
    if scenario == 'packages':
        os.environ['MEMOQUILL_M3_REIMPORT_AUDIT'] = '0'
        os.environ['MEMOQUILL_M3_ORIGINAL_COPY_AUDIT'] = '0'
    if os.environ.get('MEMOQUILL_M3_JSON_ONLY') == '1':
        json_property_import(0)
        if os.environ.get('MEMOQUILL_M3_SCREENSHOTS') == '1':
            d.capture('m3-json-records')
        d.key('alt+F4')
        return
    attach('/input/import.csv')
    click('Preview original import.csv')
    d.wait_for(lambda: shown('beginneedle') and shown('endneedle'), 'original beginning and end retained')
    click('Create import preview')
    d.wait_for(lambda: d.find('check box', 'Select record 1'), 'durable import records displayed')
    field('Title column', '2')
    field('Body column', '3')
    for index in (1, 2, 3):
        d.focus('check box', f'Select record {index}')
        d.key('space')
    click('Apply selected records')
    d.wait_for(lambda: shown('Import operation saved locally.'), 'reviewed import acknowledged')
    for title in ('First quantum record', 'Second quantum record', 'Last quantum record'):
        d.wait_for(lambda title=title: d.find('button', title), f'imported material visible: {title}')
    for query, title in (
        ('beginneedle', 'First quantum record'),
        ('middleneedle', 'Second quantum record'),
        ('endneedle', 'Last quantum record'),
    ):
        field('Search titles, tags and text', query)
        click('Search materials')
        d.wait_for(lambda title=title: d.find('button', title) and shown('matches in this batch'), f'indexed search finds {query}')
        d.wait_for(lambda: any(re.search(r'Original [0-9a-f-]{36},', d.text(node)) for node in d.nodes()), f'search locator accompanies {query}')
    click('Show all materials')
    click('Rebuild search index')
    d.wait_for(lambda: d.find('button', 'Rebuild search index').getState().contains(d.pyatspi.STATE_ENABLED), 'index rebuild control becomes available')
    field('Search titles, tags and text', 'beginneedle')
    click('Search materials')
    d.wait_for(lambda: d.find('button', 'First quantum record') and shown('matches in this batch'), 'canonical result remains available after explicit rebuild')
    click('Show all materials')
    if os.environ.get('MEMOQUILL_M3_REIMPORT_AUDIT', '1') == '1':
        reimport_and_undo()
        json_property_import()
    if os.environ.get('MEMOQUILL_M3_ORIGINAL_COPY_AUDIT', '1') == '1':
        save_original('import.csv')
        for filename in ('original.pdf', 'original.epub', 'original.docx', 'hostile.epub', 'hostile.html'):
            attach(f'/input/{filename}')
            click(f'Preview original {filename}')
            d.wait_for(lambda: shown('Original preview'), f'{filename} has an explicit passive preview')
            save_original(filename)
    else:
        results.append({'check': 'original copy', 'status': 'not_run', 'reason': 'intermediate candidate; native copy action pending'})
    if os.environ.get('MEMOQUILL_M3_PACKAGE_AUDIT', '1') == '1':
        package_roundtrips()
    else:
        results.append({'check': 'package recovery', 'status': 'not_run', 'reason': 'explicit subset run'})
    if os.environ.get('MEMOQUILL_M3_SCREENSHOTS') == '1':
        d.capture('m3-imported-records')
    d.key('alt+F4')


try:
    main()
    scope = 'JSON mapping subset' if os.environ.get('MEMOQUILL_M3_JSON_ONLY') == '1' else 'ingestion/search' + ('/reimport/JSON' if os.environ.get('MEMOQUILL_M3_REIMPORT_AUDIT', '1') == '1' else '') + ('/originals' if os.environ.get('MEMOQUILL_M3_ORIGINAL_COPY_AUDIT', '1') == '1' else '') + ('/packages' if os.environ.get('MEMOQUILL_M3_PACKAGE_AUDIT', '1') == '1' else '')
    if os.environ.get('MEMOQUILL_M3_SCENARIO', 'full') != 'full':
        scope = os.environ['MEMOQUILL_M3_SCENARIO'] + ' focused scenario'
    Path('/evidence/m3-journey.json').write_text(json.dumps({'scope': scope, 'scenario': os.environ.get('MEMOQUILL_M3_SCENARIO', 'full'), 'checks': results}, ensure_ascii=False, indent=2))
except Exception as error:
    controls = []
    for node in d.nodes():
        try:
            if node.name:
                state = node.getState()
                controls.append({'role': node.getRoleName(), 'name': node.name, 'enabled': state.contains(d.pyatspi.STATE_ENABLED), 'showing': state.contains(d.pyatspi.STATE_SHOWING)})
        except d.GLib.Error:
            pass
    Path('/evidence/m3-visible-text.json').write_text(json.dumps([{'role': node.getRoleName(), 'name': node.name, 'text': d.text(node)[:1000]} for node in d.nodes() if node.getRoleName() in ('paragraph', 'text', 'static', 'label', 'alert', 'status')], ensure_ascii=False, indent=2))
    Path('/evidence/m3-controls.json').write_text(json.dumps(controls, ensure_ascii=False, indent=2))
    Path('/evidence/m3-journey.json').write_text(json.dumps({'scope': 'ingestion subset', 'checks': results, 'failure': repr(error)}, ensure_ascii=False, indent=2))
    raise
