# Isolated course workspaces after M2

Scope: KER-127, KER-128–130. The user authorized design and implementation on
2026-11-08, then delegated the existing-data migration decision after being
shown the shared-Subject problem. The choices below are engineering decisions,
not claims of user-selected implementation details or learning efficacy.

## Intent and constraints

A course is the top-level sidebar workspace. Its nested Subjects, collections,
Units/lessons, resources, sources, presentation, drafts, responses, search,
preferences, receipts and future history form one isolation boundary. The
learner can create independent courses and switch without discarding work.
Preserve offline operation, immutable versions/originals, guest identity and
typed provider-independent services. Main M1–M8 targets X11; full visual design,
M3 ingestion, M4 study, M6 transport/schedules and executable simulators remain
outside this step. No GUI launch, screenshot, push, merge or history rewrite.

## Approaches and decision

1. Add course-owner columns to every existing table and validate every query,
   reference and uniqueness constraint. This can work but requires a large
   cross-cutting migration and leaves isolation dependent on every filter.
2. Give each course its own existing workspace database, originals and recovery
   root; put only course metadata in an application catalogue. This reuses the
   tested single-writer, version, receipt and recovery boundaries and makes
   cross-course joins impossible. Selected for this step.
3. Promote every Subject to a course automatically. M2 allows Units/resources,
   paths, tags and placements to span Subjects; automatic splitting would need
   ambiguous copying/rebinding of versions and history. Rejected as an implicit
   upgrade policy. A later explicit content-copy/split operation can be designed
   separately; it must not create linked mutable courses.

## Identity, ownership and migration (KER-128)

- A versioned native catalogue owns stable UUID course IDs and titles. IDs are
  routing identities, distinct from database-issued workspace IDs and actor IDs.
- Adopt the old application `workspace` directory in place as the first course,
  named `Imported workspace` when old data exists. Do not move, clone, rewrite,
  split or delete its database, managed originals, snapshots or recovery files.
  A fresh installation starts with one empty `My course` in that same location.
- All existing Subjects become nested organization inside that first course;
  Unit IDs remain lesson IDs. Every old placement, resource revision, source,
  composition, activity, draft, annotation, reading state and operation receipt
  stays in its original database. Future history is owned by that database too.
  No M4 history schema is invented or claimed to exist now.
- New course roots are derived natively from validated catalogue UUIDs beneath
  a private managed directory. Callers cannot supply a root/path. Originals may
  be deduplicated inside one course only; no mutable or source binding crosses
  course roots. Identical bytes in separate courses confer no shared authority.
- Publish catalogue bootstrap/create/selection changes transactionally. Exact
  create retries return the same course; conflicting retry payloads fail.
  Unknown/corrupt/newer catalogue data fails recoverably without resetting it.
  Preserve abandoned managed roots after an interrupted creation; never adopt
  unknown roots or remove user data automatically.
- Bootstrap does not upgrade or open the legacy database. Its existing schema
  checksum/migration/recovery guard runs when the course service opens. Catalogue
  failure does not modify that database. Catalogue recovery is separate from
  course recovery; recovering one course never changes another.
- Use exclusive catalogue ownership. Version 1 admits at most 32 courses/services;
  refuse further creation recoverably with budget_exceeded, without eviction or
  discarding pending requests. Drain every started writer on exit. No daemon.

## Services and UI (KER-129)

- Course catalogue access is local trusted application authority; it exposes
  metadata only. Every data-bearing native call requires an explicit course ID.
  Missing/unknown IDs fail; there is no mutable process-wide current-course
  fallback. The router resolves a service once before any await or file picker.
- Retain existing workspace/lesson DTO checks inside that selected service.
  A mutation's workspace ID must match that course database. A foreign object,
  draft, revision or receipt cannot be resolved by consulting another course.
  Search counts/cursors/snippets and shared-impact calculations use one database.
- Native authorization still constructs RequestContext and denies foreign
  webviews/origins. Existing unverified external contexts remain denied. A future
  agent grant must bind courseId + workspaceId + actor + allowed operations and
  object/range scope; it cannot route through global course selection. No tokens
  or live MCP endpoint are implemented here.
- Create application-port factories bound to immutable course IDs. Nested UI
  consumes those ports through the existing provider seam. All content, source,
  workspace preference and lesson operations capture the same course identity.
- A functional sidebar lists/selects/creates courses. Keep each visited course
  subtree mounted under its own provider/key and hide inactive subtrees. This
  retains selection, editor undo, unsaved drafts, responses, preview, reading
  state and pending exact-retry requests while switching. Inactive content is
  hidden from focus/accessibility navigation. Pending responses update their
  originating subtree only. Course titles are rendered as plain text.
- Persist the last selected course in catalogue metadata; reopening starts that
  course. Existing durable drafts and saved state reopen through current lesson
  APIs. Unsaved UI edits retain the existing close-warning behavior; no promise
  of crash durability for unsaved browser memory is added.
- Keep native focus/window details in adapters. Use current plain functional
  styling, semantic navigation and labels; no visual redesign or GUI acceptance
  is implied by DOM tests.

## Future interactive extensions (KER-130)

- Add a data-only version-1 extension manifest/state contract, separate from the
  trusted built-in registry. Preserve stable extension ID, exact manifest and
  state schema versions, course/workspace/lesson/node ownership, bounded JSON,
  declared capabilities, provenance and compatibility/migration metadata.
- Known trusted built-ins retain their existing renderer. Unknown extension
  identifiers, newer versions, unavailable capabilities or unsupported state
  versions preserve their original bounded payload and render an inert readable
  fallback. No dynamic import, eval, iframe/script handler, network request or
  native/DB/filesystem access follows from a manifest or source document.
- Capabilities describe named application operations and course-local state,
  not ambient execution permissions. A declaration is a request, not a grant.
  Deny cross-course scope and undeclared/unsupported capabilities. Never treat
  extension-produced values as independent competence or trusted assessment.
- Compare a sandboxed web worker (no DOM, explicit message bridge), a sandboxed
  frame/process with a narrow bridge, and a WASM host with explicit imports in a
  later runtime decision. A worker alone is not an authority/security boundary;
  origin/network, resource budgets, cancellation, accessibility, source review
  and host imports require qualification before executable acceptance. Current
  execution policy is `inertOnly` for all external/generated extensions.
- Migrations are reviewed application-owned deterministic transforms. No
  manifest-supplied executable migrator. Preserve old state/version and pinned
  attempt semantics; unsupported migration refuses activation and offers inert
  recovery. Actual runtime and sample simulators are excluded.

## Evidence and acceptance

Reuse `docs/research/learning-science.md`, `learning-activities.md`,
`ai-directed-learning-brief.md` and `lesson-contract-evidence.md`. This change
settles engineering isolation and contracts, not pedagogy or learning outcomes.
Current source: M2 ApplicationService owns one root/queue; organization Snapshot
is workspace-wide; Unit = lesson; source selection is registered to a service.
Graph Tier 2, project home-void-Project-Memoquill, generation
2026-11-08T11:46:58Z; relied-on source paths checked without recorded gaps.
Graph results are best effort, not an exhaustive proof.

Required checks: legacy fixture remains byte-identical through catalogue-only
bootstrap/reopen and failed bootstrap; service open/reopen preserves identities,
content and links subject to existing supported migrations; catalogue corruption,
exact create retry and unknown-ID denials; two real course services with
distinct content/search/drafts/receipts/preferences and cross-course ID denial;
service drain/restart; bound port payloads for every command; UI switching with
unsaved drafts and delayed results; extension scope/capability/version/budget
validation and opaque inert round trip; existing relevant frontend/native
regressions, TypeScript/build, fmt/Clippy. No dependency changes intended.
GUI/native visual acceptance remains not run without fresh permission.

## Implementation discretion

Use the existing Rust/SQLite/Tauri/React stack, existing owner/file protections,
and small modules. Catalogue versioning is independent of per-course schema 5.
Concrete table names, error reuse and test layout belong to implementation.
Preserve pre-existing AGENTS.md, .gitignore, workflow-skill and November 1 report
changes. Commit finished substeps locally in dev only.
