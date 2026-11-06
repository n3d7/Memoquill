"""Packaged lesson-composer smoke checks through keyboard/AT-SPI only."""

import re
from pathlib import Path


def focus_window(d, mode):
    if mode == 'wayland':
        match = re.search(r'window id (\d+)', Path('/evidence/weston.log').read_text())
        if not match:
            raise AssertionError('Nested Weston input surface is unavailable')
        d.command('xdotool', 'windowfocus', match.group(1))
        d.wait_for(lambda: 'wl_display' in Path('/evidence/app-wayland.log').read_text(), 'native Wayland protocol connection')
    else:
        window = d.command('xdotool', 'search', '--name', 'Memoquill').splitlines()[-1]
        d.command('xdotool', 'windowfocus', window)


def choose_block(d, choose, kind):
    # Editor activation/IME may create paragraphs; ordinals are not stable IDs.
    names = [n.name for n in d.nodes(d.find('combo box', 'Selected block'))]
    match = next((re.fullmatch(r'(\d+)\. ' + re.escape(kind), name) for name in names if re.fullmatch(r'(\d+)\. ' + re.escape(kind), name)), None)
    if not match:
        raise AssertionError(f'Canonical {kind} block missing from native choices: {names}')
    choose('Selected block', int(match.group(1)) - 1)


def exercise(d, click, type_in, choose, shown):
    for title in ('English lesson', 'Physics lesson'):
        type_in('New lesson title', title)
        click('Create lesson')
        d.wait_for(lambda: d.find('button', 'Edit lesson'), f'{title} has its own empty composition')
        click('Edit lesson')
        d.wait_for(lambda: d.find('button', 'Add component'), f'{title} gets a durable editing draft')
        if title == 'English lesson':
            click('Add component')  # Default component is a section.
            d.wait_for(lambda: d.find('entry', 'Section title'), 'section settings open after native semantic insertion')
            type_in('Section title', 'English-only content')
            click('Save component settings')
            d.wait_for(lambda: shown('Editing draft saved locally.'), 'section configuration is acknowledged')
            type_in('Section title', 'English unsaved form')
        else:
            assert not shown('English-only content')
            assert not shown('English unsaved form')
            choose('Lesson spacing', 0)
            click('Save lesson appearance')
            d.wait_for(lambda: shown('Editing draft saved locally.'), 'second lesson stores independent appearance')
            click('Save lesson')
            d.wait_for(lambda: shown('Lesson saved locally.'), 'second lesson receives native save receipt')
    d.focus('page tab', 'English lesson')
    d.key('space')
    d.wait_for(lambda: d.text(d.find('entry', 'Section title')) == 'English unsaved form', 'switching tabs preserves the first lesson form independently')
    click('Preview draft')
    d.wait_for(lambda: shown('Preview only. Answers here do not record learning progress.'), 'native draft preview is explicitly separate from learning history')
    click('Save lesson')
    d.wait_for(lambda: shown('Lesson saved locally.'), 'first lesson receives its own native save receipt')
    click('Close current lesson')
    d.wait_for(lambda: d.find('page tab', 'Physics lesson').getState().contains(d.pyatspi.STATE_SELECTED), 'closing one lesson activates the other lesson')
    assert not shown('English-only content')
    d.results.append({'check': 'separate lesson tabs preserve content, forms and appearance independently', 'passed': True})


def reopened(d, click, choose, shown):
    def select_lesson(title):
        # Unit map order is not a creation-order contract; native option names
        # determine the keyboard position, without Orca type-ahead interception.
        names = [node.name for node in d.nodes(d.find('combo box', 'Existing learning unit')) if node.name in ('Choose learning unit', 'English lesson', 'Physics lesson')]
        if len(names) != 3 or len(set(names)) != 3:
            raise AssertionError(f'Native lesson choices unavailable: {names}')
        choose('Existing learning unit', names.index(title))

    select_lesson('English lesson')
    click('Open lesson')
    d.wait_for(lambda: d.find('button', 'Saved lesson').getState().contains(d.pyatspi.STATE_ENABLED), 'first lesson composition survives process restart')
    click('Saved lesson')
    d.wait_for(lambda: shown('English-only content'), 'acknowledged first-lesson content survives process restart')
    select_lesson('Physics lesson')
    click('Open lesson')
    d.wait_for(lambda: d.find('page tab', 'Physics lesson').getState().contains(d.pyatspi.STATE_SELECTED), 'second lesson reopens independently after restart')
    click('Saved lesson')
    assert not shown('English-only content')
    d.results.append({'check': 'restarted second lesson has no first-lesson content', 'passed': True})
