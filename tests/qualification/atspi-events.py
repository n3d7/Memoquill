"""Record live-region AT-SPI events on the private qualification bus only."""
import json
import sys
from pathlib import Path

import pyatspi
from gi.repository import GLib

output = Path(f'/evidence/live-events-{sys.argv[1]}.jsonl')
output.write_text('')
count = 0


def record(event):
    global count
    if count >= 500:
        return
    try:
        node = event.source
        for _ in range(5):
            if node is None:
                return
            attributes = node.getAttributes()
            if any('live:' in value for value in attributes):
                row = {'type': event.type, 'role': event.source.getRoleName(),
                       'name': event.source.name, 'data': str(event.any_data),
                       'attributes': attributes}
                with output.open('a') as stream:
                    stream.write(json.dumps(row, ensure_ascii=False) + '\n')
                count += 1
                return
            node = node.parent
    except GLib.Error:
        pass  # An accessible can disappear while its event is in flight.


pyatspi.Registry.registerEventListener(record, 'object:')
pyatspi.Registry.start()
