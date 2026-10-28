"""Verify Orca's generated speech after its buffered debug file is closed.

This checks presentation requests, not audible playback or speech quality.
"""
import json
import sys
from pathlib import Path

mode = sys.argv[1]
log = Path(f'/evidence/orca-{mode}.log').read_text()
speech = '\n'.join(line for line in log.splitlines() if 'SPEECH OUTPUT:' in line)
expected = {
    'requested structural result': 'Draft checked: Quantum physics',
    'result dialog context': 'Operation result',
    'Rust validation error': 'Title must contain 1–128 characters.',
    'picker cancellation feedback on requested reading': 'No file selected. Your edits are unchanged.',
    'readable formula source': r'E=\hbar\omega',
    'passive code content': 'def double(x):',
    'selected file name and safety disclosure': 'Selected memoquill-example.md. Contents were not read or executed.',
}
checks = [{'check': name, 'passed': value in speech} for name, value in expected.items()]
Path(f'/evidence/speech-{mode}.json').write_text(json.dumps(checks, indent=2))
if not all(check['passed'] for check in checks):
    raise AssertionError(f'Expected Orca presentation missing: {checks}')
print(json.dumps({'mode': mode, 'speechChecksPassed': len(checks)}))
