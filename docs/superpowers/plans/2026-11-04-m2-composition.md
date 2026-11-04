# M2 composition implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Superpowers subagent-driven-development; one implementer then task review, no parallel implementation workers. Follow TDD and commit completed substeps only.

**Goal:** deliver the remaining M2 services, replaceable frontend and isolated offline lesson constructor, then qualify all executable acceptance criteria.
**Architecture:** existing Unit is lesson identity; immutable composition/activity revisions and durable authoring drafts use the existing single writer. Typed frontend ports isolate native transport; a trusted owned renderer serves authoring and preview. No mutable state is shared between lesson tabs.
**Tech stack:** current Rust/rusqlite/Tauri and React/TypeScript/Vitest, no new dependency.
**Spec:** docs/architecture/lesson-composition.md, final user isolation correction; docs/planning/m2-contracts.md and live KER-118/119/120/116 acceptance.

## Global constraints

- Work on existing dev checkout as explicitly requested, preserve unrelated AGENTS/.agents/.gitignore and two tooling reports. No push/merge/PR/release.
- User 2026-11-04 authorizes continuing all M2 implementation immediately; this supersedes the architecture hold and redundant approval pauses, not security/acceptance gates.
- No GUI launch or screenshots. Native/physical GNOME/human Orca acceptance must remain explicitly unverified if prohibited.
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
**Interfaces:** composer receives workspaceId, Unit and LessonPort. Each tab mounts an independent controller keyed by Unit. New lesson uses existing createUnit service. Pins resources through existing readDocument exact revision. Authoring activity definitions use typed Task 1 service. Render with typed view model/actions; preview separate disposable responses/assistance by node occurrence.

- [ ] Write failing component tests for add/edit/remove/reorder/nest/configure, undo/redo, preview/save/reopen, two isolated lessons, unknown inert fallback, failed apply/exact retry, stale async response, keyboard/focus/semantic order and four subjects.
- [ ] Implement stack/columns/section/content/source/activity/button/status registry with closed typed fields and safe rendering, forms for all initial activity families and bounded condition/event configuration. No arbitrary JS/HTML/CSS/URLs.
- [ ] Implement durable draft edit/preview/apply and recoverable save conflicts. Shared render path for draft preview and saved lesson; unavailable M4 study actions clearly report preview mode/capability unavailable.
- [ ] Integrate accessible independent lesson tabs and per-lesson appearance/source selection; document editor remains reusable primitive. No hidden cross-tab mutations or global style effects.
- [ ] Run scoped frontend suite and build, commit completed KER-120 substep after review.

## Task 4: KER-113/114/115/116 integrated qualification

**Files:** focused regressions plus docs/quality acceptance ledger, m2 execution, handoff and task report. Existing native harness only for static changes; do not launch GUI.

- [ ] Map KER-99 and every child acceptance to current implementation, tests and remaining human evidence; do not infer completion from status.
- [ ] Execute existing full `npm run check` once on final reviewed code; targeted changed-code security/dependency/license checks appropriate to actual edits, no external private-code transfer.
- [ ] Independent whole-change review with exact diff, fix concrete findings with regression tests, rerun only impacted gates.
- [ ] Record passed/failed/skipped/unavailable gates. Update Linear to evidence-supported states; no Done for mandatory unrun native/physical acceptance.
- [ ] Commit qualification/docs substep, preserve unrelated changes, no push/merge. If human GUI acceptance remains unavailable, state M2 is locally implemented/qualified to the permitted extent, not fully accepted.

## Execution rulings

- Existing dev checkout is the explicitly requested workspace; a new feature branch/worktree would violate the requested substep commit destination.
- Keep full architecture target in scope. First-slice tests alone do not satisfy KER-120; remaining registry/types and all executable criteria belong to the same milestone.
- No new literature retrieval is needed for implementing already-researched contracts; verify library-specific uncertainties only when encountered.
