---
name: memoquill-task-workflow
description: Use when researching, planning, implementing, reviewing, or resuming a substantial authorized Memoquill task. Preserve the current phase and distinguish design decisions from implementation authority.
---

# Memoquill Task Workflow

Adapted from Scryntic task workflow; Copyright 2026 n3d7, Apache-2.0.
See `../../NOTICE` and `../../LICENSES/Apache-2.0.txt` from this skill directory.

1. Read repository `AGENTS.md` and retain the user's goal, phase, constraints, and existing authorization. Review-only work stays read-only. Foundation research/planning does not authorize application implementation.
2. Read `docs/HANDOFF.md` when continuing, then relevant documents through `docs/README.md` when available. Do not reload the entire documentation tree. Missing planned documents are deliverables, not evidence of prior decisions.
3. Inspect current branch/HEAD, Git status, and relevant diffs/new files. Preserve user work. Follow discovery/coverage rules in `AGENTS.md` for code relationships; read non-code instructions directly.
4. When Linear matters, inspect the Memoquill project, issue, dependencies, and duplicates. Never use another product's project. If unavailable, use local issue definitions, record the exact limitation, and do not invent remote status.
5. Perform the authorized task with maintainable artifacts. Research requires verified sources, evidence quality/applicability, and a distinction between findings and hypotheses. Implementation requires current-source prerequisite checks and concrete acceptance criteria.
6. Preserve subject independence, offline operation, original sources, learning history, and scoped agent access. Ordinary reversible research/documentation decisions are authorized by the Foundation assignment. Record significant decisions as ADRs; seek clarification only for consequential ambiguity or missing authority.
7. Validate the actual deliverable. Research needs traceable sources and consistent decisions; docs need reference/scope checks; implementation needs risk-appropriate tests. Record checks, failures, and gaps. Missing/failing required validation means incomplete work.
8. Review relevant diffs, new files, and Git status. Use the reporting skill for a substantial result or interruption. Update Linear only within existing authorization and to evidence-supported states. Commit, push, and publication require explicit requests.

If interrupted or nearing a context limit, preserve findings before starting more work. Update `docs/HANDOFF.md` with the authorized task, completed/incomplete work, artifacts, verified repository/tracker state, blockers, and one concrete next step. A handoff is context, not proof its facts remain current.
