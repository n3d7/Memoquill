# M3 completion implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development; execute task-by-task with independent review and bounded evidence.

**Goal:** Complete KER-100/KER-101, commit substeps and open dev-to-main PR, no merge.
**Architecture:** Course-scoped typed writer services own durable jobs/revisions/index
and consistent backup; native adapters own selected files and staged new-course restore.
**Tech Stack:** Existing Rust/SQLite/Tauri/React stack, qualified bounded archive/parser dependencies.
**Spec:** ../specs/2026-11-09-m3-completion-design.md

## Global Constraints

Linux/X11 only. Preserve all existing data/IDs/history and unrelated working-tree
changes. No GUI/screenshots, remote content upload, implicit fetch, active input,
M4/M6 runtime or non-X11 work. User authorizes local commits, push dev and dev→main
PR only; no merge, history rewrite or force push. Keep artifacts English.
Reconcile every import/backup item; prefixes/subsets never claim whole-source/full recovery.

## Review Focus

- Restart/lost response/retry must not duplicate applied material or lose outcomes.
- Changed-source reimport and undo must preserve human edits and history.
- Derived index must remain course-scoped/rebuildable across ordinary content writes.
- Active WAL and assets must form a consistent snapshot under writer/lease barrier.
- Corrupt/future/traversal/bomb archives must never mutate any live course.

### Task 1: Complete durable ingestion and indexed discovery

**Files:** Own new src-tauri/src/workspace/ingestion/ and migrations/006_ingestion.sql,
existing workspace service/migrations/content/search/types integration, native
source/authoring commands and corresponding src/application ports/types/native
adapters; focused import/source UI and tests. May modify relevant files under src
and src-tauri; preserve unrelated files. No packaging implementation in this task.
**Interfaces:** Public typed ApplicationService methods for job create/read/list,
preview, cancel, reviewed subset apply, reimport lineage and guarded undo; exact
DTOs documented in task report for Task 2/3. Requests use existing workspace/
course/operation/revision authority. Rebuildable FTS5 behind existing SearchPage.

- [ ] Read existing content/source/writer/receipt patterns and acceptance sources;
  define exact DTOs and record implementation choices in the task report.
- [ ] Write failing tests first for durable job/restart/cancel/retry/partial apply,
  invalid/oversized input, exact duplicate/unchanged reimport, human-edit conflicts,
  guarded undo and scoped Unicode/code search/rebuild/scale.
- [ ] Implement bounded extraction/mapping and persisted reconciled job items,
  atomic reviewed apply/retry, explicit reimport/undo previews; preserve provenance.
- [ ] Integrate course-bound IPC/ports and useful offline preview/mapping/job UI.
- [ ] Run focused tests then normal frontend build/coverage and native fmt/Clippy/
  full tests offline; expect pass, record actual skipped/failures.
- [ ] Commit only owned task files with a concrete English message; return SHA,
  changed paths, checks and unresolved acceptance. Controller obtains fresh review.

### Task 2: Deliver portable packages and verified new-course recovery

**Files:** Own new workspace/packages modules/fixtures, package manifest/NDJSON and
safe archive/native adapters; integrate course manager/service, commands/ports/UI,
tests and docs/architecture/package-format.md. Preserve Task 1 APIs and revisions.
**Interfaces:** Task 1 schema automatically included by native snapshot; portable
export declares subset/section versions. New course restore returns explicit report
and course ID; every mutation uses typed trusted native/service authority.

- [ ] Inspect Task 1 report and final schema, existing snapshot/recovery/catalogue
  patterns; document exact format/inclusion and history-extension contracts.
- [ ] Write failing G1/G2 roundtrip tests including active WAL, source assets,
  revisions/lessons/jobs/drafts/preferences, missing originals and incomplete subset.
- [ ] Implement serialized DB backup + captured immutable-asset leases, manifest/
  hash/size validation, bounded ZIP/JSON/NDJSON, readable projections and safe
  staged NEW-course restore. Never replace a live course or follow archive paths.
- [ ] Exercise corruption/version/traversal/symlink/duplicate/bomb/failed-restore
  rollback and relevant writer/asset races with real databases/files.
- [ ] Add useful UI for selected destination/source, explicit export kind/review/
  completeness and restore report; run relevant frontend/native gates.
- [ ] Commit owned files; return SHA, format contract/checks and limitations.
  Controller obtains independent task review before further implementation.

### Task 3: Qualify formats and close integration/acceptance gaps

**Files:** Relevant source handler/native UI integration, focused fixtures/tests,
docs/quality/m3-qualification.md, docs/architecture/content-ingestion.md,
dependency license inventory/notices and task reports/HANDOFF.
**Interfaces:** Reuse Task 1/2 DTOs; add qualified handlers only behind their existing
contract. Explicit original-only capability is a supported honest fallback.

- [ ] Record verified parser/security/license/packaging gate and view/extract matrix
  for PDF/EPUB/DOCX/HTML and initial formats; add passive-original access where needed.
- [ ] Run representative source original-only/extraction/coverage fixtures, no
  execution/network, and beginning/middle/end search plus keyboard-oriented UI tests.
- [ ] Verify every KER-100/101 criterion against source/tests; fix software gaps,
  record remaining human-only M7 evidence separately, never imply GUI acceptance.
- [ ] Refresh dependency notices from actual locked sources. Run final full checks,
  independent whole-change review and fix material findings with RED/GREEN.
- [ ] Commit verified qualification/docs/integration substeps.

### Task 4: Publish the completed M3 PR without merge

**Files:** English PR title/body using repository template and pr skill; publication
receipt/task report/HANDOFF. Exclude unrelated pre-existing tooling changes.

- [ ] Inspect final dev diff against fetched main, commit list, remote branch state,
  existing PR duplicates and acceptance evidence. Preserve all original SHAs.
- [ ] Push dev without force; create/update one clear dev→main PR with behavior,
  scope, G1/G2/format capabilities, verification and honest manual limitations.
- [ ] Read back PR base/head/body/open status and remote CI where exposed. Do not
  merge or enable auto-merge. Report exact PR URL and any pending checks.
