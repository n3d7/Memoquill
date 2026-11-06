"""Resume only post-restart checks in an already-authored synthetic profile."""
import importlib.util
import json
from pathlib import Path
import sys


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


d = load('native_driver', 'atspi-journey.py')
c = load('composer_checks', 'm2-composer.py')
mode = sys.argv[1]


def click(name):
    d.focus('button', name)
    d.key('space')


def choose(name, index):
    d.focus('combo box', name)
    d.key('alt+Down')
    d.key('Home')
    for _ in range(index):
        d.key('Down')
    d.key('Return')
    d.key('Escape')


def shown(value):
    return any(value in d.text(node) for node in d.nodes())


try:
    d.wait_for(lambda: d.find('button', 'Native quantum notes'), 'authored profile reopens without replaying passed authoring checks')
    c.focus_window(d, mode)
    click('Quantum physics')
    click('Native quantum notes')
    d.wait_for(lambda: 'Native quantum provenance' in d.text(d.find('entry', 'Learning document')), 'canonical prose survives reopen')
    d.wait_for(lambda: shown('Native anchored note'), 'anchored note survives reopen')
    c.choose_block(d, choose, 'math')
    d.wait_for(lambda: r'E=\hbar\omega' in d.text(d.find('entry', 'Formula source')), 'authored math source survives reopen by canonical block type')
    d.focus('entry', 'Formula source')
    d.read_from_focus()
    click('Preview original m2-original.html')
    d.wait_for(lambda: shown('<script>never_execute_original()</script>'), 'original remains passive and unchanged after reopen')
    c.reopened(d, click, choose, shown)
    if mode == 'wayland':
        assert d.find('button', 'Close').queryAction().doAction(0)
    else:
        d.key('alt+F4')
finally:
    Path(f'/evidence/m2-reopen-{mode}.json').write_text(json.dumps(d.results, ensure_ascii=False, indent=2))
