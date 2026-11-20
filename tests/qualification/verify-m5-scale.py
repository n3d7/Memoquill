#!/usr/bin/env python3
"""Explicit scale-fixture checks through the existing verification engine.

No ordinary verification profile creates this dataset. Run with
MEMOQUILL_SCALE_MODE=pilot|full and --profile affected. Retained SSD output is
synthetic and exclusive-create; an existing fixture is never overwritten.
"""
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/verification'))
import verify

mode = os.environ.get('MEMOQUILL_SCALE_MODE')
if mode not in ('pilot', 'full'):
    raise SystemExit('MEMOQUILL_SCALE_MODE must be pilot or full')
original_plan = verify.plan
original_config = verify.integrations.nextest_config


def nextest_config(folder):
    config, report = original_config(folder)
    # Only this explicit generation helper needs a longer test-body budget.
    if mode == 'full':
        config.write_text(config.read_text().replace('terminate-after = 2', 'terminate-after = 59'))
    return config, report


def plan(profile, changed, frontend=None, rust=None):
    if profile == 'affected':
        check = original_plan(profile, changed, rust='generate_scale_fixture')[0][0]
        check['name'] = 'm5-scale-' + mode
        check['command'] += ['--run-ignored', 'only']
        check['timeout'] = 3600 if mode == 'full' else 600
        return [check], 'Explicit synthetic scale generation; admission/performance/GUI remain separate.'
    return original_plan(profile, changed, frontend, rust)


verify.plan = plan
verify.integrations.nextest_config = nextest_config
raise SystemExit(verify.main())
