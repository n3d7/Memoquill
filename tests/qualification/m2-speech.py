"""Verify Orca presentation requests; this is not human listening evidence."""
import json
from pathlib import Path
import sys

mode = sys.argv[1]
speech = '\n'.join(line for line in Path(f'/evidence/orca-{mode}.log').read_text().splitlines() if 'SPEECH OUTPUT:' in line)
expected = {
    'subject authoring control': 'New subject',
    'material editor': 'Learning document',
    'authored formula source': r'E=\hbar\omega',
    'accessible requested result': 'Operation result',
    'durable preferences acknowledgement': 'Reading preferences saved',
}
checks = [{'check': name, 'passed': text in speech} for name, text in expected.items()]
Path(f'/evidence/m2-speech-{mode}.json').write_text(json.dumps(checks, indent=2))
if not all(check['passed'] for check in checks):
    raise AssertionError(checks)
print(json.dumps({'mode': mode, 'speechChecksPassed': len(checks)}))
