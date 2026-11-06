# M2 composition implementation plan

Current user authority,2026-11-06: docs/Linear portability planning only; application implementation/builds/GUI checks are paused. When resumed, qualify main M1–M8 on X11 only and retain [per-stage portability](../../architecture/platform-portability.md). Non-X11 matrices, including Wayland, are deferred under ADR-10/KER-122–126; historical unchecked criteria below are not converted into passes. Review revised X11 exit criteria before closing M2.

> **For agentic workers:** Superpowers execution with the explicit user override below. Follow focused TDD and commit completed substeps only; avoid broad repeated checks and per-task review dispatch.

**Goal:** deliver the remaining M2 services, replaceable frontend and isolated offline lesson constructor, then qualify all executable acceptance criteria.
**Architecture:** existing Unit is lesson identity; immutable composition/activity revisions and durable authoring drafts use the existing single writer. Typed frontend ports isolate native transport; a trusted owned renderer serves authoring and preview. No mutable state is shared between lesson tabs.
**Tech stack:** current Rust/rusqlite/Tauri and React/TypeScript/Vitest, no new dependency.
**Spec:** docs/architecture/lesson-composition.md, final user isolation correction; docs/planning/m2-contracts.md and live KER-118/119/120/116 acceptance.

## User process override — 2026-11-05

The user explicitly asks to conserve usage: run focused regressions for changed behavior and type checks for changed interfaces now; defer full suites/build/lint/security and one whole-change review to the end of M2. Do not repeat broad source reads, long reports or per-task review/re-review cycles. Existing concrete data-loss/isolation/security findings still get fixed and covering tests. Preserve test evidence in short logs, no completion claims for deferred gates. This overrides overlapping Superpowers per-task review requirements and prior plan check cadence below. No feature/safety boundary is removed.

## Global constraints

- Work on existing dev checkout as explicitly requested, preserve unrelated AGENTS/.agents/.gitignore and two tooling reports. No push/merge/PR/release.
- User 2026-11-04 authorizes continuing all M2 implementation immediately; this supersedes the architecture hold and redundant approval pauses, not security/acceptance gates.
- User follow-up authorizes opening the app when needed for visual checks and requesting screenshots from the user. Never capture screenshots ourselves. Native/physical GNOME/human Orca evidence remains separately verified; no host reconfiguration.
- No M3 parser, real M4 attempts/evaluator, M6 transport/grants or M8 motivation implementation. M2 activity preview is simulated and cannot write study history.
- One rooted composition tree: 1 MiB JSON, 256 nodes, depth 12, 64 children/slot, 16 KiB props/node, 256 bindings, 64 source bindings. Conditions: 32 nodes/depth 4, no scripts/paths/loops. Batch: 64 operations, 2 MiB envelope, 512 revision closure. Draft caps 20 and 32 MiB per actor/Unit.
- Different editable lesson tabs cannot alias mutable composition, drafts, response/assistance/preferences/schedule/progress. No future linked-tab seam. Immutable source bytes may be reused; pins never auto-follow target heads.
- Shared semantic operations, strict validation, exact bases, preview/apply hash, atomic durable receipt, idempotent retry and retained conflicts. Never silently truncate or discard unknown payloads.

## Review focus

1. Two Units referencing the same immutable content retain isolated edits and preview responses.
2. Lost apply acknowledgement returns exactly the prior receipt; same request ID with different payload fails.
3. Persisted/unknown/deep/cyclic objects cannot bypass validation at read/apply/restore.
4. Async replies after tab switch cannot overwrite another Unit's draft or preview; unresolved saves retain exact retry.
5. Existing document editor save/selection/annotations/source protections survive adapter extraction.

## Task 1: KER-119 typed lesson persistence and native service

**Files:** create `src-tauri/src/workspace/lessons/` with focused types/validation/storage/tests modules and migration `005_lessons.sql`; modify workspace migration registration/mod/service, native command registration/capability lists where necessary; create `src/lessons/types.ts` and `src/lessons/nativeLessonPort.ts` (transport adapter only). Do not edit existing frontend components or contentBridge.

**Interfaces:** publish TypeScript `Composition`, `LessonNode`, `Binding`, `Condition`, `LessonOperation`, `LessonDraft`, `LessonPreview`, `SavedComposition`, `LessonReceipt`, `LessonPort`. camelCase serialized DTOs. `Composition` fields follow spec (`contractVersion:1`, `unitId`, `rootNodeId`, `nodes`, `bindings`, `sourceBindingRefs`, `appearance`, `authoringProvenance`); revision metadata belongs SavedComposition. Node fields `nodeId,typeId,typeVersion,props,slots,on,visible?`. Registry types stack,columns,section,content,source,activity,button,status. Initial activity families recall,singleChoice,constructed,procedure are immutable definitions outside layout props.

`LessonPort`: `read(unitId):Promise<SavedComposition|null>`, `createDraft({workspaceId,unitId,operationId,expectedHeadRevision:string|null}):Promise<LessonDraft>`, `editDraft({workspaceId,unitId,draftId,expectedDraftRevision:number,operationId,operations:LessonOperation[]}):Promise<LessonDraft>`, `previewDraft({workspaceId,unitId,draftId,expectedDraftRevision:number}):Promise<LessonPreview>`, `applyDraft({workspaceId,unitId,draftId,expectedDraftRevision:number,expectedHeadRevision:string|null,previewHash:string,operationId}):Promise<LessonReceipt>`, `discardDraft(...)`, `getReceipt(operationId)`. `LessonDraft` exposes `id,unitId,revision,composition,baseHeadRevision`; preview exposes `draftId,draftRevision,composition,hash,warnings`; saved exposes `id,unitId,composition`; receipt exposes `operationId,saved` and mappings/committed metadata. Additional typed read-revision/activity/source methods allowed where required by spec. Publish the types early and report exact interface decisions before UI task begins.

- [ ] Write meaningful failing Rust tests first: independent Units/drafts, scoped refs, cycles/unreachable/duplicate children/limits/unknown fields, stale preview/apply, same-ID exact retries and changed payload, reopen migration/history preservation, unsupported preservation/inert read, activity occurrence pins.
- [ ] Run targeted tests and observe missing feature failure. Implement pure schema/tree/registry checks, semantic edit operations including configureInteraction, source/activity reference pinning and bounded migrations.
- [ ] Implement schema5 durable drafts/immutable revisions, dependency closure and transactions using existing recovery/receipt conventions; read validates persisted data. No caller-supplied trust or raw DB/path API.
- [ ] Integrate via existing bounded ApplicationService queue and typed Tauri commands with existing native-owned RequestContext/capability boundary.
- [ ] Run new Rust tests and fmt/Clippy; test current-schema upgrade/reopen and lost replies. Report exact tests, limitations, changed files and no false claim of real-study implementation.
- [ ] Commit completed KER-119 substep only; no unrelated files. Write task report and review package.

## Task 2: KER-118 frontend application/identity port

**Files:** `src/application/` port/native/test adapter/identity modules and tests; existing frontend bridges/hooks as needed; `src/MaterialPanel.tsx`, `src/LibraryApp.tsx` only where extraction necessary. Do not modify Rust or lesson persistence DTOs.
**Interfaces:** retain existing contentBridge exported signatures as compatibility facade; native invoke lives in adapters only. A typed content application port and local guest identity/session port supply deterministic substitutes. LessonPort from Task 1 remains the lesson application boundary; components receive it rather than import its native adapter.

- [ ] Write failing tests that inject a deterministic port, replace a representative lesson/material view without native transport, and preserve failed-save/retry/dirty protections.
- [ ] Extract native calls, scenario/controller coordination and presentational boundaries without rewriting domain/storage. Keep guest actor stable per local profile/workspace and distinct from future grants; no fake login.
- [ ] Centralize reusable accessible controls/tokens, preserving existing semantics and tests. Avoid unrelated screen redesign.
- [ ] Run relevant frontend tests and build, inspect no native IPC import in presentation, and commit completed KER-118 substep.

## Task 3: KER-120 isolated composer and renderer

**Files:** `src/lessons/` controller, renderer, composer, tabs, tests; root LibraryApp integration and lesson styles. Consume Task 1 DTOs and Task 2 ports; no SQL/domain migration changes.
**Interfaces:** composer receives workspaceId, Unit and LessonPort. Each tab mounts an independent controller keyed by Unit. New lesson uses existing createUnit service. Pins resources through Unit-scoped readResource with an exact revision. Authoring activity definitions use typed Task 1 service. Render with typed view model/actions; preview separate disposable responses/assistance by node occurrence.

- [ ] Write failing component tests for add/edit/remove/reorder/nest/configure, undo/redo, preview/save/reopen, two isolated lessons, unknown inert fallback, failed apply/exact retry, stale async response, keyboard/focus/semantic order and four subjects.
- [ ] Implement stack/columns/section/content/source/activity/button/status registry with closed typed fields and safe rendering, forms for all initial activity families and bounded condition/event configuration. No arbitrary JS/HTML/CSS/URLs.
- [ ] Implement durable draft edit/preview/apply and recoverable save conflicts. Shared render path for draft preview and saved lesson; unavailable M4 study actions clearly report preview mode/capability unavailable.
- [ ] Integrate accessible independent lesson tabs and per-lesson appearance/source selection; document editor remains reusable primitive. No hidden cross-tab mutations or global style effects.
- [ ] Run scoped frontend suite and build, commit completed KER-120 substep after review.

## Task 4: KER-113/114/115/116 integrated qualification

**Files:** focused regressions plus docs/quality acceptance ledger, m2 execution, handoff and task report. Existing native harness may be used only with screenshot capture disabled; open a verified candidate for user-provided screenshots when needed.

- [ ] Map KER-99 and every child acceptance to current implementation, tests and remaining human evidence; do not infer completion from status.
- [ ] Execute existing full `npm run check` once on final reviewed code; targeted changed-code security/dependency/license checks appropriate to actual edits, no external private-code transfer.
- [ ] Independent whole-change review with exact diff, fix concrete findings with regression tests, rerun only impacted gates.
- [ ] Record passed/failed/skipped/unavailable gates. Update Linear to evidence-supported states; no Done for mandatory unrun native/physical acceptance.
- [ ] Commit qualification/docs substep, preserve unrelated changes, no push/merge. If human GUI acceptance remains unavailable, state M2 is locally implemented/qualified to the permitted extent, not fully accepted.

## Execution rulings

- Existing dev checkout is the explicitly requested workspace; a new feature branch/worktree would violate the requested substep commit destination.
- Keep full architecture target in scope. First-slice tests alone do not satisfy KER-120; remaining registry/types and all executable criteria belong to the same milestone.
- No new literature retrieval is needed for implementing already-researched contracts; verify library-specific uncertainties only when encountered.

## Task 5: KER-119 staged resource read follow-up (execute before Task3)

**Reason:** interface verification found readActivity/readSource resolve durable Pending definitions but readDocument is published-only; reopened staged resource previews otherwise cannot render. This completes the existing preview/recovery contract, not a new feature.
**Files:** existing workspace lessons storage/read helper, ApplicationService/native command/ACL registration; src/lessons/types.ts and nativeLessonPort.ts; focused Rust/adapter tests. No migration, renderer, new dependency or source parser.
**Interfaces:** add `LessonPort.readResource(unitId: string, revisionId: string): Promise<SavedDocument | null>` using existing contentBridge SavedDocument type; native `lesson_read_resource` query. Resolve the requested Unit's durable Pending::Resource revision first, then authorized immutable published dependency closure. Do not use ephemeral frontend caches or global unscoped reads. Preserve exact revision ID/document before and after apply.

- [ ] Write failing test: stage a new resource with createResourceRevision, reopen database/service, read through Unit-scoped public port and get exact staged ID/document while global readDocument remains unpublished.
- [ ] Write failing tests: another Unit cannot read the staged pin; apply then same scoped read returns same immutable published revision; later edits do not alter it. Denial does not disclose source metadata.
- [ ] Run focused tests RED, implement read-only resolver through existing actor/service/native boundaries and matching typed frontend adapter.
- [ ] Align catalog response.update value bound with actual scalar validation:4096 UTF-8 bytes (catalog.rs currently16384 maxLength). Include ASCII4096/4097 and multibyte boundary regression/metadata; document byte semantics so tools do not treat code-point length as sufficient. Keep event/action/state descriptors consistent.
- [ ] Run focused Rust/adapter tests, fmt/Clippy/build as affected; commit only this independently testable substep and record exact logs/report.
- [x] Final integration review covered this interface under the user's reduced-cadence override. Fix1c1fb43 additionally closes published-resource pins in validated durable drafts before apply; the persistence child is Done.

Execution order refinement: Task1 fix/review → Task2 → Task5 → Task3 → Task4. Task2 does not depend on resource rendering, so its frontend boundary extraction can proceed first. Task3 must consume the real scoped readResource method.

## Execution checkpoint — 2026-11-06

Tasks1/2/3/5 implementation and the single integration-review fix wave are complete;
the historical checklists above are not instructions to repeat completed work.
Task4's local qualification checkpoint records passed X11 and failed/unrun platform
gates in docs/quality/m2-qualification.md; M2 acceptance remains incomplete.
Additional startup fix 5c72fdb resolves the user-confirmed Linux/NVIDIA GBM white window.
The user next authorizes finishing KER-97/M1.1; no M3 work is authorized.
