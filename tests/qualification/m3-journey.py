"""Native M3 ingestion journey; later package/recovery checks are separately recorded.

Runs only in the isolated qualification desktop. No direct IPC/SQL/DOM mutation.
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

os.environ.setdefault('MEMOQUILL_CAPTURE_SCREENSHOTS', os.environ.get('MEMOQUILL_M3_SCREENSHOTS', '0'))
spec = importlib.util.spec_from_file_location('native_driver', Path(__file__).with_name('atspi-journey.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)
results = d.results


def shown(value):
    return any(value in d.text(node) for node in d.nodes())


def click(name):
    d.focus('button', name)
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
    click('Choose source file')
    d.wait_for(lambda: any(node.getRoleName() in ('dialog', 'file chooser') for node in d.nodes()), 'source chooser opened')
    d.key('ctrl+l')
    d.command('xdotool', 'type', '--clearmodifiers', path)
    d.key('Return')
    d.wait_for(lambda: shown('Selected:') or d.open_enabled(), 'source path validated')
    if not shown('Selected:'):
        click('Open')
    d.wait_for(lambda: shown('Selected:'), 'source selected through native chooser')
    field('Attribution', 'Original synthetic M3 qualification fixture')
    field('License', 'Original project test fixture')
    click('Attach selected source')
    d.wait_for(lambda: d.find('button', 'Preview original import.csv'), 'original attached with provenance')


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
    results.append({'check': 'ingestion native subset', 'passed': True, 'packageRecovery': 'not_run'})
    if os.environ.get('MEMOQUILL_M3_SCREENSHOTS') == '1':
        d.capture('m3-imported-records')
    d.key('alt+F4')


try:
    main()
    Path('/evidence/m3-journey.json').write_text(json.dumps({'scope': 'ingestion subset; full M3 qualification pending', 'checks': results}, ensure_ascii=False, indent=2))
except Exception as error:
    Path('/evidence/m3-journey.json').write_text(json.dumps({'scope': 'ingestion subset', 'checks': results, 'failure': repr(error)}, ensure_ascii=False, indent=2))
    raise
