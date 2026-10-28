"""Bounded native discovery probe. No application data or host bus is accessed."""
import json
import sys
import time
from pathlib import Path
import pyatspi

mode = sys.argv[1]
application = None
for attempt in range(200):
    desktop = pyatspi.Registry.getDesktop(0)
    for child in desktop:
        if 'memoquill' in child.name.lower():
            application = child
            break
    if application is not None:
        break
    time.sleep(.1)
if application is None:
    raise RuntimeError('Memoquill did not appear on the isolated AT-SPI bus')

records = []
def visit(node, depth=0):
    if len(records) >= 1500 or depth >= 30:
        return
    try:
        record = {'depth': depth, 'role': node.getRoleName(), 'name': node.name}
        if node.queryText:
            try:
                text = node.queryText()
                record['text'] = text.getText(0, min(text.characterCount, 300))
            except NotImplementedError:
                pass
        records.append(record)
        for child in node:
            visit(child, depth + 1)
    except Exception as error:
        records.append({'error': type(error).__name__})

time.sleep(3)
visit(application)
Path(f'/evidence/atspi-{mode}.json').write_text(json.dumps(records, ensure_ascii=False, indent=2))
print(json.dumps({'mode': mode, 'application': application.name, 'accessibleNodes': len(records)}))
