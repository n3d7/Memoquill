---
name: memoquill-task-reporting
description: Use when recording substantial Memoquill research, documentation, implementation, review, or investigation, including blocked or incomplete work and explicit requests for a task report.
---

# Memoquill Task Reporting

Adapted from Scryntic task reporting; Copyright 2026 n3d7, Apache-2.0.
See `../../NOTICE` and `../../LICENSES/Apache-2.0.txt` from this skill directory.

Write a factual, proportional record before finishing substantial work or handing over an interruption. Skip trivial questions and routine status updates. Reporting grants no new execution or publication authority.

## Report

- Write in English under `docs/task-reports/`, using `YYYY-MM-DD-HHMMSSZ-<topic>-<slug>.md` with actual UTC time. Preserve earlier reports. Prefer a few paragraphs or 20–40 lines and links to detailed evidence.
- Reports are repository artifacts unless the user requests local-only reports. Never include secrets or sensitive payloads, even in ignored files.
- Lead with scope and actual outcome: complete, incomplete, or blocked. Include the Linear issue/link when relevant.
- Record work, material decisions/reasons, changed paths, and pre-existing user work separately.
- Record validation actually performed; distinguish passed, failed, skipped, unavailable, and not run. Scientific sources, technical tests, usability evaluation, and learning-outcome evidence are different evidence types.
- State unmet criteria, uncertainties, blockers, and one concrete continuation step. Use “none” when nothing remains; do not invent follow-up work.
- When relevant, record branch/HEAD and distinguish local edits, commits, push, PR/CI, tracker writes, and release state. Never infer one from another.
- Link the report in the final response. If saving is blocked, explain the limitation and give essential facts directly.
- Record an actual-use inventory of skills, MCP integrations, tools and plugins, including failed/unavailable attempts when material. Distinguish reading a skill from executing its workflow, and discovery from use. Include the concise inventory in the final response as required by `AGENTS.md`.

## Current checkpoint

Update `docs/HANDOFF.md` for substantial work and unfinished handovers. Keep it short, normally 20–40 lines, with actual UTC time, current authorized phase, outcomes, paths, validation, repository/tracker state, incomplete work, blockers, one next step, and resume pointers. Retain the original mission when intermediate requests steer the work.

On continuation, read the latest relevant report/checkpoint and refresh drift-prone facts. Do not rerun unchanged checks merely to populate a report.
