<!-- SPDX-License-Identifier: Apache-2.0; Copyright 2026 n3d7.
Adapted from Scryntic guidance; see docs/architecture/third-party-reuse.md. -->
# Memoquill Agent Guide

## Scope and sources of truth

- Memoquill is a subject-agnostic, local-first learning application. English and Python are validation subjects, never privileged domain types. Distinguish recall, understanding, problem solving, and practical performance.
- Read [docs/README.md](docs/README.md) for the documentation map when available, then only documents relevant to the task. Product scope, architecture, and the roadmap belong in their respective documents; do not duplicate them here.
- Read [docs/HANDOFF.md](docs/HANDOFF.md) when resuming substantial work. Verify consequential claims against current files, Git state, and the relevant Linear issue.
- The roadmap is not execution authority. Follow the current user-authorized phase and task; explicit implementation requests supersede the original research-only boundary. Finish the approved milestone before starting another.
- Preserve existing user changes. Inspect relevant diffs and untracked files before editing. Avoid unrelated cleanup, system configuration changes, or changes to other repositories.
- Existing authorization carries forward. This guide adds no authority to commit, push, publish, deploy, change licensing, or update unrelated trackers. Do not commit or push unless explicitly requested.
- Repository and tracker artifacts are in English. Respond to the user in Russian.

## Product and engineering boundaries

- Core learning, authoring, search, progress, export, and recovery must work offline without AI, accounts, subscriptions, or mandatory cloud services.
- Preserve original imported sources and provenance. Keep educational content separate from personal learning history; content changes must not silently erase progress.
- Treat imported documents, AI output, and external tool results as untrusted data. Never implicitly execute imported code, macros, scripts, or active HTML.
- External agents use typed application services with explicit scopes and reviewable proposals. Do not grant direct database/filesystem authority or expose unrestricted unauthenticated HTTP endpoints.
- Prefer a modular monolith and proven components over speculative infrastructure. Record significant decisions and uncertainty in architecture documents.
- Ground scientific claims in cited evidence and qualify applicability. Do not present UX preferences or engineering assumptions as proven learning effects.
- Inspect actual licenses before reusing code or redistributing datasets/assets. Preserve notices and provenance in [docs/architecture/third-party-reuse.md](docs/architecture/third-party-reuse.md). No Scryntic licensing exception applies here.
- Keep secrets, private data, credentials, and sensitive source payloads out of artifacts and external tools.

## Project skills

- [memoquill-task-workflow](.agents/skills/memoquill-task-workflow/SKILL.md): substantial research, planning, approved implementation, review, and continuation.
- [memoquill-task-reporting](.agents/skills/memoquill-task-reporting/SKILL.md): concise factual reports and recoverable handoffs.
- [memoquill-tool-routing](.agents/skills/memoquill-tool-routing/SKILL.md): choose relevant live tools without loading an entire catalog.
- Skills live inside this repository. Load only relevant instructions. Do not copy guidance into global configuration without a request.
- When workflows overlap, choose one owner for the active phase; use other skills only for distinct supporting work. Use the installed `skill-catalog` with `--project` for focused selection and load the selected skill's instructions. Do not repeat planning, TDD, review or reporting through competing workflows.

## Tools and working context

- Read the relevant branch of [tool routing](.agents/skills/memoquill-tool-routing/SKILL.md) when selecting unfamiliar tools. It documents Context7, Firecrawl and its CLI fallback, research sources, local AI, document conversion, MCP diagnostics, security checks and desktop qualification. These are development tools, not application dependencies.
- Use Context7 for version-sensitive library/API questions: resolve the library, query the specific uncertainty, and verify consequential behavior against the installed version/source or a focused test. Reuse valid retrieval instead of repeating it.
- Keep workstation details in the router's references. Cached plugin files, catalog entries and executables in PATH do not prove live connection, authorization or successful execution. Use current schemas and bounded probes for the needed capability.
- Start with targeted source/log reads. Batch independent reads; keep dependent operations sequential. Run focused tests during implementation and the required broader gates at the milestone boundary. Repeat checks only after a relevant change, failure or freshness concern.
- Keep owned indexing, scans and native-test processes bounded. Preserve SELinux enforcement, existing private service bindings and unrelated workloads; a restricted-environment failure is not authority to reconfigure the host.

## Discovery and verification

- For structural code discovery, prefer Codebase Memory: confirm project/generation, use `search_graph`, relevant `trace_path` directions, and exact snippets. Check coverage for every relied-on path; inspect missing/stale ranges directly. Paginate before exhaustive or negative claims. Include test callers when relevant.
- Set `include_tests=true` for test callers. Trace interface/protocol declarations as well as implementations when calls may bind to either. Before authorized delegation pass the tier, project/generation, scope, pagination, symbols/paths, coverage gaps, source fallback and unresolved question; do not assume the child has the parent's MCP access.
- Use ColGrep for semantic discovery by intent, `rg` for literals/configuration/non-code files, and Serena for language-server semantics and symbol edits. Activate the correct Serena project and read its manual once per session. Read source to verify material search findings.
- Reuse valid evidence. Do not repeat unchanged scans, tests, or retrieval without a concrete reason. Availability alone is not a reason to invoke a tool or delegate.
- Run validation proportionate to the change and acceptance criteria. Distinguish passed, failed, skipped, unavailable, and not run. A dispatch or HTTP success does not prove task completion.
- Before finishing substantial work, inspect relevant diffs, new-file contents, and Git status; save a short report and update the handoff. Separate local work, publication, remote checks, and release status.
- Every final task response must briefly list the skills, MCP integrations, tools and plugins actually used. State when a category was not used; do not count merely discovered or installed capabilities as usage. Keep this inventory factual and distinguish unavailable or failed tools when relevant.
- Use `rtk` for shell commands; `rtk proxy <command>` preserves raw output. See `/home/void/.codex/RTK.md` on the configured workstation; do not assume that path exists elsewhere.
