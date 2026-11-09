# Course Workspaces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Deliver isolated offline courses and inert future-extension contracts for KER-127/128–130.
**Architecture:** One existing workspace database/root per course; a separate private versioned catalogue routes explicit course IDs. Course-bound application ports and mounted UI subtrees retain async/draft isolation.
**Tech Stack:** Existing Rust, SQLite, Tauri 2, TypeScript, React and Vitest; no new dependencies.
**Spec:** ../specs/2026-11-08-course-workspaces-design.md

## Global Constraints

- Main M1–M8 targets X11; full visual design and other platforms remain deferred.
- No GUI launch, screenshots, push, merge or history rewrite. Local commits in dev only.
- Preserve existing workspace bytes/IDs and pre-existing user changes.
- No live MCP, generated-code runtime, new ingestion, real study or schedules.
- Native callers supply course IDs, never storage paths or authority contexts.

## Review Focus

- Catalogue bootstrap failure must never reset legacy data (Task 1 fixture).
- Same operation/object IDs in two courses must not resolve across them (Tasks 1/2).
- File selection and responses completing after a switch retain original course (Task 2 captured ports/handler).
- Corrupt/newer catalogue and abandoned roots are preserved, never silently recreated (Task 1).
- Extension declarations cannot grant execution or cross-course authority (Task 3).

### Task 1: KER-128 catalogue and course ownership

**Files:** Create `src-tauri/src/workspace/courses.rs` and `courses/tests.rs`; modify `workspace/mod.rs`; preserve migrations 001–005 unchanged.
**Interfaces:** `CourseManager::open(app_data: PathBuf) -> Result<Self>`; `list() -> Result<CourseCatalogue>`; `create(operation_id: Uuid, title: String) -> Result<CourseCatalogue>`; `select(course_id: Uuid) -> Result<CourseCatalogue>`; `service(course_id: Uuid) -> Result<ApplicationService>`; `shutdown().await`.

- [x] Write tests for legacy in-place adoption/byte preservation, catalogue restart/exact retry/conflict, invalid/unknown/newer/corrupt catalogue, two independent service roots and drain/restart. Literal user fixtures, no generated expectation.
- [x] Run `rtk proxy bash tests/qualification/with-test-env.sh cargo test --manifest-path src-tauri/Cargo.toml --lib workspace::courses`; Expected: RED for missing course API.
- [x] Implement catalogue version 1 with private owner, transactionally stored metadata, native-derived roots, bounded services and shutdown. Existing root remains untouched until requested by service.
- [x] Run the same focused command; Expected: all course tests PASS. Inspect diff and commit only Task 1 files/design/plan locally as `feat(courses): preserve existing workspace in isolated course catalogue`.

### Task 2: KER-129 native routing and functional course navigation

**Files:** Create `src-tauri/src/course_commands.rs`, `src/courses/{types.ts,nativeCoursePort.ts,CourseApp.tsx,CourseApp.test.tsx,nativeCoursePort.test.ts}`; modify native handlers `lib.rs`, `authoring.rs`, `attachments.rs`, `lesson_commands.rs`, command capability config; modify native port factories, `main.tsx` and minimal styles.
**Interfaces:** Native catalogue list/create/select require trusted local context; data calls require `courseId`. `createNativeCoursePorts(courseId)` returns `ApplicationPorts` with fixed scope; `CourseApp` owns catalogue and mounted course trees.

- [x] Write failing native tests for explicit route/unknown/missing IDs, foreign workspace/object/draft/receipt/search denial. Write failing port-payload tests and DOM switching tests with unsaved edits and delayed results.
- [x] Run focused native and `rtk proxy npm test -- src/courses`; Expected: RED on missing routing/UI.
- [x] Wire manager startup/drain, explicit per-call service resolution, immutable port factories and semantic course sidebar. Keep nested lesson UI and close warnings; no default mutable routing.
- [x] Run focused tests, then frontend suite, Rust suite, build, fmt/Clippy; Expected: PASS. Commit this substep locally after reviewing new files and diffs.

### Task 3: KER-130 data-only extensions and completion

**Files:** Create `src/extensions/{contracts.ts,contracts.test.ts,ExtensionPlaceholder.tsx}` and native contract if persistence needs it; update relevant architecture docs, HANDOFF, qualification/report and this ledger.
**Interfaces:** Versioned manifest/state admission returns a bounded inert representation with diagnostics. Course/workspace/lesson scope, capability declarations, compatibility, provenance and migration metadata are explicit. Built-in renderer unchanged; external/generated components always inert.

- [x] Write failing tests for opaque preservation, incompatible versions, scope/capability denial and budgets; Expected: RED before implementation.
- [x] Implement contracts/admission and accessible inert fallback. Document runtime alternatives and migration/recovery gates; do not activate any executable mechanism.
- [x] Run extension and existing trusted-renderer tests plus relevant build/native gates; Expected: PASS. Review all task diffs with one fresh reviewer, resolve consequential findings, record limitations, update docs and commit locally.

## Execution ledger

- 2026-11-08: User delegated migration choice. Selected in-place adoption as one course and separate roots; automatic Subject splitting rejected because shared data has no unique ownership. Cost: old Subjects remain nested until a separately designed explicit split/copy operation.
- Execution is inline in dev as requested; no worktree/branch change. Proportionate checks and existing research reused. Plan-helper workspace files, if needed, stay ignored; tracked ledger is the recovery record.

Completion: all tasks implemented and independently reviewed; final counts in docs/task-reports/2026-11-08-161306Z-post-m2-course-integration.md. Minor extension diagnostic-branch coverage deferred. No publication/GUI.
