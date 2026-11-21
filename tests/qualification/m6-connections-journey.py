"""Test the installed app and packaged MCP bridge inside the private X11 display."""
import importlib.util
import json
import os
from pathlib import Path
import select
import subprocess
import traceback
import uuid

out = Path('/evidence')
spec = importlib.util.spec_from_file_location('driver', Path(__file__).with_name('atspi-journey.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)
d.capture = lambda *_: None
children = []
failure = None
counter = 0


def check(condition, description):
    if not condition:
        raise AssertionError(description)
    d.results.append({'check': description, 'passed': True})


def focus(name):
    for node in d.nodes():
        if node.name == name:
            try:
                if node.queryComponent().grabFocus():
                    return node
            except NotImplementedError:
                pass
    raise LookupError(name)


def activate(name):
    d.wait_for(lambda: focus(name).getState().contains(d.pyatspi.STATE_ENABLED), name, record=False)
    d.key('space')


def field(name, value):
    d.wait_for(lambda: focus(name).getState().contains(d.pyatspi.STATE_FOCUSED), name, record=False)
    d.key('ctrl+a')
    d.command('xdotool', 'type', '--clearmodifiers', value)
    d.key('Tab')


def request(bridge, method, params):
    global counter
    counter += 1
    bridge.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': counter, 'method': method, 'params': params}) + '\n')
    bridge.stdin.flush()
    if not select.select([bridge.stdout], [], [], 12)[0]:
        raise AssertionError('MCP reply timed out')
    reply = json.loads(bridge.stdout.readline())
    assert reply['id'] == counter, reply
    return reply


def tool(bridge, name, arguments=None):
    reply = request(bridge, 'tools/call', {'name': name, 'arguments': arguments or {}})
    assert 'result' in reply, reply
    return reply['result']


def listeners():
    ports = set()
    for path in ('/proc/net/tcp', '/proc/net/tcp6'):
        if Path(path).is_file():
            ports.update(row.split()[1] for row in Path(path).read_text().splitlines()[1:] if row.split()[3] == '0A')
    return ports


try:
    with (out / 'm6-bridge.log').open('w') as log:
        bridge = subprocess.Popen(['memoquill', 'mcp'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log, text=True)
        children.append(bridge)
    init = request(bridge, 'initialize', {'protocolVersion': '2025-11-25', 'capabilities': {},
                                         'clientInfo': {'name': 'Synthetic X11 qualification', 'version': '1'}})
    check(init['result']['protocolVersion'] == '2025-11-25', 'Packaged stdio negotiates the legacy protocol')
    bridge.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    bridge.stdin.flush()
    closed = tool(bridge, 'memoquill_connection')
    check(closed['isError'] and 'app_not_running' in closed['content'][0]['text'], 'Closed application returns actionable error')
    before = listeners()
    with (out / 'm6-app.log').open('w') as log:
        app = subprocess.Popen(['memoquill'], stdout=log, stderr=log)
        children.append(app)
    d.wait_for(lambda: d.find('button', 'Create course'), 'Installed desktop is ready')
    activate('Connections')
    activate('Enable local connections')
    pending = tool(bridge, 'memoquill_connection')['structuredContent']
    check(pending['connection'] == 'pending', 'Host receives a challenge only after explicit opt-in')
    denied = tool(bridge, 'memoquill_capabilities')
    check(denied['isError'] and 'permission_denied' in denied['content'][0]['text'], 'Pending client cannot read its course')
    activate('Refresh connections')
    field('Pairing challenge', pending['challenge'])
    activate('Published course structure and normalized document text')
    approval = next(node.name for node in d.nodes() if node.name and node.name.startswith('Approve for '))
    activate(approval)
    d.wait_for(lambda: tool(bridge, 'memoquill_connection')['structuredContent']['connection'] == 'approved',
               'Trusted desktop approves the matching bridge')
    capabilities = tool(bridge, 'memoquill_capabilities')['structuredContent']
    course_id = capabilities['courseId']
    check(capabilities['permissions'] == {'content': True, 'progress': False, 'sources': False,
                                          'responses': False, 'originals': False, 'drafts': False, 'delegateAdditions': False}, 'Sensitive reads are separate opt-ins')
    data = tool(bridge, 'memoquill_structure_get', {'kind': 'units'})
    check(isinstance(data['structuredContent']['items'], list), 'Approved client reads a typed course page')
    hidden = tool(bridge, 'memoquill_progress_get', {'id': '863b57be-7248-4cdf-8eb7-97a5bfaf4765'})
    check(hidden['isError'] and 'permission_denied' in hidden['content'][0]['text'], 'Content grant denies learner progress')
    check(listeners() == before, 'Enabling Connections does not open an Internet listener')
    field('New course', 'Synthetic independent course')
    activate('Create course')
    d.wait_for(lambda: d.find('button', 'Synthetic independent course'), 'User creates and selects a second course')
    check(tool(bridge, 'memoquill_capabilities')['structuredContent']['courseId'] == course_id,
          'Course selection does not retarget the existing grant')
    activate('Connections')
    revoke = next(node.name for node in d.nodes() if node.name and node.name.startswith('Revoke bridge '))
    activate(revoke)
    denied = tool(bridge, 'memoquill_capabilities')
    check(denied['isError'], 'Revoked bridge cannot disclose further material')
    missing_grant = tool(bridge, 'memoquill_lesson_draft', {
        'unitId': str(uuid.uuid4()), 'ownerKey': str(uuid.uuid4()),
        'action': {'kind': 'create', 'operationId': str(uuid.uuid4())},
    })
    check(missing_grant['isError'] and 'permission_denied' in missing_grant['content'][0]['text'],
          'An unapproved client cannot create a lesson draft')
    pending = tool(bridge, 'memoquill_connection')['structuredContent']
    activate('Refresh connections')
    field('Pairing challenge', pending['challenge'])
    activate('Published course structure and normalized document text')
    activate('Create and edit unpublished course and lesson drafts')
    approval = next(node.name for node in d.nodes() if node.name and node.name.startswith('Approve for '))
    activate(approval)
    d.wait_for(lambda: tool(bridge, 'memoquill_connection')['structuredContent']['connection'] == 'approved',
               'Trusted desktop explicitly grants draft authoring')
    field('New lesson title', 'Synthetic MCP draft lesson')
    activate('Create lesson')

    def find_draft_unit():
        reply = tool(bridge, 'memoquill_structure_get', {'kind': 'units'})
        return next((unit for unit in reply['structuredContent']['items']
                     if unit['title'] == 'Synthetic MCP draft lesson'), None)

    d.wait_for(find_draft_unit, 'The new lesson is visible through the authorized course service')
    unit = find_draft_unit()['id']
    owner = str(uuid.uuid4())

    def draft(action, owner_key=owner):
        return tool(bridge, 'memoquill_lesson_draft', {
            'unitId': unit, 'ownerKey': owner_key, 'action': action,
        })

    created = draft({'kind': 'create', 'operationId': str(uuid.uuid4())})['structuredContent']['value']
    draft_id = created['id']
    check(created['composition']['authoringProvenance']['kind'] == 'assisted',
          'An MCP draft records external authoring provenance')
    foreign = draft({'kind': 'read', 'draftId': draft_id}, str(uuid.uuid4()))
    check(foreign['isError'], 'Another owner cannot inspect a known draft ID')
    preview = draft({'kind': 'preview', 'draftId': draft_id, 'revision': 1})['structuredContent']
    check(preview['kind'] == 'preview', 'The owner reads a structured inert preview')
    published = tool(bridge, 'memoquill_lesson_get', {'id': unit})
    check(published['isError'], 'The draft does not create a published lesson head')
    discard = draft({'kind': 'discard', 'draftId': draft_id, 'revision': 1, 'operationId': str(uuid.uuid4())})
    check(discard['structuredContent']['kind'] == 'discarded', 'The owner discards its unpublished draft')
    next_draft = draft({'kind': 'create', 'operationId': str(uuid.uuid4())})['structuredContent']['value']
    activate('External lesson drafts')
    activate('Refresh external drafts')
    activate('Inspect draft ' + next_draft['id'])
    activate('Apply reviewed draft')
    activate('Confirm publication')
    d.wait_for(lambda: tool(bridge, 'memoquill_lesson_get', {'id': unit}).get('structuredContent', {}).get('unitId') == unit,
               'Trusted review publishes the exact external lesson')
    def course_draft(action):
        return tool(bridge, 'memoquill_course_draft', {
            'ownerKey': owner, 'action': action,
        })

    def course_group(title, chunks):
        current = tool(bridge, 'memoquill_structure_get', {'kind': 'units'})['structuredContent']
        group = str(uuid.uuid4())
        created = course_draft({'kind': 'create', 'draftId': group, 'title': title,
                                'baseRevision': current['revision']})['structuredContent']
        assert created['status'] == 'editing', created
        for index, operations in enumerate(chunks, 1):
            action = {'kind': 'append', 'draftId': group, 'revision': index,
                      'operationId': str(uuid.uuid4()), 'operations': operations}
            advanced = course_draft(action)['structuredContent']
            assert advanced['revision'] == index + 1, advanced
            assert course_draft(action)['structuredContent'] == advanced
        submitted = course_draft({'kind': 'submit', 'draftId': group, 'revision': len(chunks) + 1,
                                  'operationId': str(uuid.uuid4())})['structuredContent']
        assert submitted['status'] == 'submitted', submitted
        validation = course_draft({'kind': 'validate', 'draftId': group,
                                   'revision': submitted['revision']})['structuredContent']
        assert validation['valid'], validation
        return group, submitted['revision'], validation['hash']

    def organization(command_type, **fields):
        return {'type': 'organization', 'command': {'type': command_type, **fields}}

    subject_id, resource_id, first_id, second_id, collection_id, path_id = (str(uuid.uuid4()) for _ in range(6))
    course_id, revision, digest = course_group('Reviewed course fixture', [
        [organization('createSubject', id=subject_id, title='Synthetic astronomy'),
         organization('createCollection', id=collection_id, subjectId=subject_id, title='Observation modules'),
         organization('createResource', id=resource_id, title='Shared observation text'),
         {'type': 'document', 'resourceId': resource_id, 'expectedRevision': 0,
          'document': {'schemaVersion': 1, 'blocks': [
              {'id': str(uuid.uuid4()), 'content': {'type': 'code', 'text': 'Synthetic observing checklist'}}]}}],
        [organization('createUnit', id=first_id, title='Observe', objective='Record observations',
                      goalIds=[], resourceIds=[resource_id]),
         organization('createUnit', id=second_id, title='Compare', objective='Compare observations',
                      goalIds=[], resourceIds=[resource_id]),
         organization('place', id=str(uuid.uuid4()), collectionId=collection_id,
                      target={'kind': 'unit', 'id': first_id}),
         organization('createPath', id=path_id, title='Observation path', steps=[
             {'id': str(uuid.uuid4()), 'unitId': first_id}, {'id': str(uuid.uuid4()), 'unitId': second_id}]),
         organization('relate', id=str(uuid.uuid4()), kind='prerequisite', **{'from': first_id, 'to': second_id})],
    ])
    absent = tool(bridge, 'memoquill_resource_read', {'id': resource_id})
    check(absent['isError'], 'A submitted course group does not create live content')
    denied = course_draft({'kind': 'apply_delegated', 'draftId': course_id, 'revision': revision,
                           'hash': digest, 'operationId': str(uuid.uuid4())})
    check(denied['isError'] and 'approval_required' in denied['content'][0]['text'],
          'Default grant cannot delegate live application')
    activate('Course draft proposals')
    activate('Refresh course proposals')
    activate('Inspect Reviewed course fixture')
    activate('Review next operations')
    activate('Apply reviewed course group')
    activate('Confirm course group')
    published = lambda: tool(bridge, 'memoquill_resource_read', {'id': resource_id})
    d.wait_for(lambda: 'Synthetic observing checklist' in published().get('structuredContent', {}).get('text', ''),
               'Trusted approval atomically publishes the reviewed course group')
    unit_rows = tool(bridge, 'memoquill_structure_get', {'kind': 'units'})['structuredContent']['items']
    check({first_id, second_id}.issubset({row['id'] for row in unit_rows}),
          'Reviewed group publishes both linked course modules')

    revoke = next(node.name for node in d.nodes() if node.name and node.name.startswith('Revoke bridge '))
    activate(revoke)
    check(tool(bridge, 'memoquill_capabilities')['isError'], 'Revocation closes the ordinary draft grant')
    pending = tool(bridge, 'memoquill_connection')['structuredContent']
    activate('Refresh connections')
    field('Pairing challenge', pending['challenge'])
    activate('Published course structure and normalized document text')
    activate('Create and edit unpublished course and lesson drafts')
    activate('Delegate safe course additions (32 operations)')
    approval = next(node.name for node in d.nodes() if node.name and node.name.startswith('Approve for '))
    activate(approval)
    d.wait_for(lambda: tool(bridge, 'memoquill_connection')['structuredContent']['connection'] == 'approved',
               'Desktop explicitly delegates bounded additions')
    delegated_id, delegated_revision, delegation_hash = course_group('Delegated fixture', [
        [organization('createSubject', id=str(uuid.uuid4()), title='Synthetic linguistics')],
    ])
    apply = {'kind': 'apply_delegated', 'draftId': delegated_id, 'revision': delegated_revision,
             'hash': delegation_hash, 'operationId': str(uuid.uuid4())}
    added = course_draft(apply)['structuredContent']
    check(added['status'] == 'applied' and added['receipt']['delegated'], 'Delegated additions commit after policy checks')
    check(course_draft(apply)['structuredContent'] == added, 'Delegated retry recovers the original receipt')
    activate('Refresh course proposals')
    activate('Inspect Delegated fixture')
    activate('Undo group additions')
    activate('Confirm undo')
    def subjects():
        return tool(bridge, 'memoquill_structure_get', {'kind': 'subjects'})['structuredContent']['items']
    d.wait_for(lambda: all(row['title'] != 'Synthetic linguistics' for row in subjects()),
               'Trusted compensation archives untouched additions')
    windows = d.command('xdotool', 'search', '--onlyvisible', '--pid', str(app.pid), '--name', 'Memoquill').splitlines()
    d.command('xdotool', 'windowactivate', '--sync', windows[-1])
    d.key('alt+F4')
    check(app.wait(timeout=15) == 0, 'Application shuts down cleanly')
    ended = tool(bridge, 'memoquill_connection')
    check(ended['isError'], 'An active bridge cannot continue after the desktop closes')
    closed = tool(bridge, 'memoquill_connection')
    check(closed['isError'] and 'app_not_running' in closed['content'][0]['text'], 'Shutdown ends the broker and the bridge cannot open storage')
    check(not (Path(os.environ['XDG_RUNTIME_DIR']) / 'memoquill-mcp' / 'broker.sock').exists(),
          'The owned runtime socket is removed on normal shutdown')
except Exception as error:
    failure = str(error) or type(error).__name__
    traceback.print_exc()
finally:
    for child in reversed(children):
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
    (out / 'm3-journey.json').write_text(json.dumps({
        'scenario': 'mcp', 'checks': d.results, 'failure': failure, 'screenshots': False,
    }, indent=2))
if failure:
    raise SystemExit(1)
