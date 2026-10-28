# Linear backlog mapping

Roadmap authority: [roadmap.md](roadmap.md). Dedicated project: [Memoquill — Foundation V1](https://linear.app/kerigan/project/memoquill-foundation-v1-05143f79bda3), `P-KER-3`, Kerigan team (`KER`). No unrelated project/issue was modified. Project, seven milestones and twelve issues were successfully created and their returned IDs/URLs recorded below. Remote read-back verification is recorded in the final report. KER-97 is now In Review with a local qualification candidate; KER-98 remains unstarted in Backlog. The remaining issues are outside this execution. Local docs are uncommitted; repository links do not imply they are published.

Seven meaningful milestones contain twelve substantial issues. Platform feasibility/durability, ingestion/recovery, session/scheduling, and MCP transport/authoring/authority risks are split where independent acceptance prevents an unmanageable combined ticket. MCP includes rich authoring and explicitly delegated reversible autonomy, not only read/propose. No time estimates or target dates are invented.

| Local key | Milestone | Linear | Dependencies | Priority |
| --- | --- | --- | --- | --- |
| M1.1 | M1 | [KER-97](https://linear.app/kerigan/issue/KER-97/m11-qualify-the-fedora-desktop-and-accessibility-baseline) | none | High |
| M1.2 | M1 | [KER-98](https://linear.app/kerigan/issue/KER-98/m12-establish-durable-workspace-and-application-service-boundaries) | M1.1 | High |
| M2 | M2 | [KER-99](https://linear.app/kerigan/issue/KER-99/m2-deliver-custom-subjects-material-authoring-and-navigation) | M1.2 | High |
| M3.1 | M3 | [KER-100](https://linear.app/kerigan/issue/KER-100/m31-build-preserved-source-ingestion-reimport-and-indexed-discovery) | M2 | High |
| M3.2 | M3 | [KER-101](https://linear.app/kerigan/issue/KER-101/m32-implement-portable-packages-and-verified-full-recovery) | M3.1 | High |
| M4.1 | M4 | [KER-102](https://linear.app/kerigan/issue/KER-102/m41-deliver-activity-authoring-feedback-and-durable-study-sessions) | M2 | High |
| M4.2 | M4 | [KER-103](https://linear.app/kerigan/issue/KER-103/m42-add-transparent-recall-scheduling-and-scoped-assessments) | M4.1 | High |
| M5 | M5 | [KER-104](https://linear.app/kerigan/issue/KER-104/m5-complete-honest-progress-personalization-and-session-continuity) | M3.2, M4.2 | Medium |
| M6.1 | M6 | [KER-105](https://linear.app/kerigan/issue/KER-105/m61-ship-opt-in-scoped-mcp-reads-over-local-stdio) | M3.2, M5 | High |
| M6.2 | M6 | [KER-106](https://linear.app/kerigan/issue/KER-106/m62-deliver-rich-mcp-draft-authoring-previews-and-resumable-jobs) | M6.1 | High |
| M6.3 | M6 | [KER-107](https://linear.app/kerigan/issue/KER-107/m63-enforce-reviewed-and-delegated-autonomous-mcp-application) | M6.2 | High |
| M7 | M7 | [KER-108](https://linear.app/kerigan/issue/KER-108/m7-qualify-and-prepare-the-fedora-foundation-v1-release) | M1.2, M2, M3.1, M3.2, M4.2, M5, M6.3 | High |

## Evidence extension

On 2026-10-27 the user authorized evidence-driven improvements. KER-102, KER-103, KER-104 and KER-106 received additive refinements for reusable activity phases, suitable feedback, assistance provenance and rich MCP authoring. No milestones, dependencies, priorities or implementation status were changed. The exact refinements are included below; [evidence portfolio](../research/learning-activities.md) and [Consensus audit](../research/consensus-review-audit.md) explain their basis.

## Milestone identifiers

- M1: `aedf6b42-a376-471d-93be-8a675e1dc0f6` — M1 — Desktop foundation and durable services
- M2: `b5390794-6a7c-4ecb-912c-fe05f941aa6a` — M2 — Custom subjects, authoring and navigation
- M3: `b6e057bb-3333-4d80-b7ff-29466ce150a3` — M3 — Ingestion, discovery and data ownership
- M4: `6ad90b1e-ad9a-4777-8b30-d3b40585182e` — M4 — Activities, feedback and recall review
- M5: `833208a0-ff09-462d-93bb-3eca7f4af125` — M5 — Progress, personalization and continuity
- M6: `0f995224-c144-4824-817f-5064e06c82f7` — M6 — Capable MCP authoring and delegated autonomy
- M7: `49e541a8-570d-40e7-8ea4-f93b0b48651a` — M7 — Fedora release qualification

## Exact issue definitions

### M1.1 — Qualify the Fedora desktop and accessibility baseline

Purpose
Resolve the highest stack risk before domain expansion.

Scope
Build the minimal Tauri 2/Rust/React/TypeScript shell and an actual Fedora RPM; exercise representative reading/editor/math/code, focus, IME, dialogs and typed IPC. Freeze supported Fedora/runtime/toolchain versions and measured baseline.

Deliverables
Version-pinned shell/build, small representative fixture, packaged test evidence, ADR-02 gate decision and documented bounded fallback.

Dependencies
None. See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Install and launch RPM offline on a clean supported Fedora baseline; a dev run alone is insufficient.
- Keyboard and Orca/AT-SPI can read/edit the representative flow and return focus correctly; record names/roles/status and math/code limitations.
- Verify Wayland, IME, file dialogs, theme/text scaling and no remote frontend/implicit network.
- If the hard gate fails, resolve or revise ADR-02 before expanding domain implementation.

Required tests
Packaged manual keyboard/Orca/IME smoke, IPC/capability negative checks, build/install/launch checks; record environment and unrun cases.

Significant risks
WebKitGTK/AT-SPI/runtime regressions; framework/library claims do not prove packaged accessibility.

Exclusions
No full learning engine, AI runtime, other platform build, or publication.

Supporting documentation
- `docs/architecture/decisions.md`
- `docs/research/technical-options.md`
- `docs/research/educational-ux.md`
- `docs/quality/verification-strategy.md`

Implementation was subsequently authorized by the user. Source, RPM and supporting documents remain local/uncommitted; no commit, push, PR, merge or release is authorized. This update concerns Memoquill only; M1.2 has not started.

#### Execution status — 2026-10-28

KER-97 was successfully moved to **In Review**, with candidate evidence appended and project/milestone/priority/dependency preserved on read-back. Actual RPM and 35 native interaction/7 Orca-presentation checks per X11 and Wayland backend pass. Manual physical Fedora/GNOME/Orca acceptance remains before Done; KER-98 remains Backlog/unstarted.

See the [candidate baseline and manual checklist](../quality/fedora-desktop-baseline.md), [review report](../task-reports/2026-10-28-103031Z-m1-desktop-qualification.md) and [evidence snapshot](../quality/m1-desktop-evidence.json). Remote issue contains the detailed execution summary; original acceptance criteria remain intact. No other issue/project was updated in this continuation.

### M1.2 — Establish durable workspace and application-service boundaries

Purpose
Give every later UI/import/agent operation one validated durable service path.

Scope
Rust domain/application/persistence modules, typed command errors/principals, SQLite single writer and migrations, asset staging/operation journal, stable IDs/settings schemas, local workspace/single-instance handling.

Deliverables
Service/repository contracts, migration/rollback fixtures, asset consistency protocol, local diagnostics and minimal fast checks.

Dependencies
M1.1 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Enable/verify foreign keys on every connection and qualify selected WAL/synchronous/busy settings.
- Asset file and parent-directory durability precedes committed references; crash failpoints create at worst recoverable orphan assets.
- Failed migration/disk-full/busy operations leave the prior workspace usable; no imported SQL/scripts execute.
- UI has no raw DB/filesystem authority; service types reserve scopes/revisions/idempotency for future MCP.

Required tests
Rust unit/integration and focused property/fault tests for identity, transactions, migration/version rejection, single-writer lock, fsync/rename/commit failpoints.

Significant risks
DB/filesystem atomicity, hardware/filesystem assumptions, accidental UI storage bypass.

Exclusions
No server, sync, full profile/account system, or generic event-sourcing framework.

Supporting documentation
- `docs/architecture/technical-design.md`
- `docs/architecture/data-model.md`
- `docs/architecture/ai-mcp-integration.md`
- `docs/quality/verification-strategy.md`

Foundation documents and the M1.1 source candidate are published in Memoquill main through PR #1. KER-98 remains planned. Its preparation notes are local/uncommitted; the current user instruction authorises preparation and then stopping, not implementation or another publication.

#### Preparation status — 2026-10-28

KER-98 preparation is complete, with no implementation. Its Backlog state, High priority, M1 milestone and KER-97 dependency remain unchanged; KER-97 is still In Review. Remote preparation notes clarify the patched SQLite engine/runtime check, bounded single writer, migration snapshots, asset publication ordering, operation receipts and failure testing.

The detailed [M1.2 preparation](m1-2-workspace-preparation.md) contains the proposed first slice and G0–G6 sequence. These local notes do not start the ticket, close manual M1.1 acceptance or authorise code/dependency/PR changes.

### M2 — Deliver custom subjects, material authoring, and navigation

Purpose
Make arbitrary educational material useful before adding review automation.

Scope
Blank subjects/goals, arbitrary collections/shared placements, units/paths/prerequisite and related links, versioned app-owned document schema/editor adapter, managed attachments/original view, tags/basic search/navigation.

Deliverables
Accessible subject workspace, resource/revision/annotation services, non-subject-specific forms, four contrasting model fixtures.

Dependencies
M1.2 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Journey A creates quantum physics without a template; English/Python/cooking use identical domain contracts.
- Moving/reusing collections/resources preserves IDs/references/history; reject containment/prerequisite cycles with recoverable errors.
- Code/math/Unicode/tables/media captions survive canonical round trips; unsupported content retains source/warning.
- Keyboard/Orca navigation, safe paste/rendering, focus return, save status and reading resumption work.

Required tests
T01/T10 domain/property/component checks, editor/schema migration and source-anchor fixtures, representative offline UI flows.

Significant risks
Flexible organization burden; editor schema lock-in; rendered active content; math/screen-reader support.

Exclusions
No graph canvas, marketplace, mandatory templates, executable programs, or fixed English/Python categories.

Supporting documentation
- `docs/product/learning-model.md`
- `docs/product/user-journeys.md`
- `docs/architecture/data-model.md`
- `docs/architecture/technical-design.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

### M3.1 — Build preserved-source ingestion, reimport, and indexed discovery

Purpose
Convert existing information into manageable learning material without recreating it or losing originals.

Scope
Staged text/Markdown/CSV/JSON handlers, mapping/preview, source/license metadata, hashes/duplicates, job states/retry/partial apply, conflict-aware reimport, FTS5, PDF/attachment feasibility gate.

Deliverables
Versioned handler contracts, import history/reconciliation, reimport/undo previews, attributable search/anchors, explicit PDF gate result.

Dependencies
M2 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Journey B preserves bytes/attribution, reconciles every item outcome and creates no silent duplicates on retry/unchanged reimport.
- Human edits/annotations/progress survive changed-source imports; uncertain anchors/conflicts remain visible.
- Untrusted formats/archives/URLs cannot execute or implicitly fetch; limits/cancellation keep UI usable.
- PDF extraction/viewer passes a maintained-parser/security/package gate or remains attachment-only with usable originals and honest status.
- Multilingual/code search is rebuildable and returns stable scoped IDs; test beginning/middle/end with keyboard/Orca at scale.

Required tests
T04/T06/T08/T10 format/Unicode/fuzz boundary cases, malformed/oversize/traversal/bomb input, crash/cancel/disk-full, duplicate/reimport conflicts, paired PDF extraction/fallback fixtures.

Significant risks
Lossy extraction, unstable upstream keys, parser complexity/security/licenses, inaccessible virtualized results.

Exclusions
No OCR/EPUB/full HTML crawl, semantic auto-merge, implicit external downloads, or AI requirement.

Supporting documentation
- `docs/architecture/content-ingestion.md`
- `docs/architecture/data-model.md`
- `docs/product/user-journeys.md`
- `docs/quality/verification-strategy.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

### M3.2 — Implement portable packages and verified full recovery

Purpose
Make user ownership and recovery independently verifiable.

Scope
Versioned ZIP/JSON/NDJSON package manifest, readable projections/originals/attribution, native DB+asset backup, asset leases, validation/new-workspace restore, history-section extension contracts.

Deliverables
Documented format/inclusion matrix, export/import adapters, complete/incomplete backup UI, restoration report and fixtures.

Dependencies
M3.1 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- G1 native full restore preserves canonical content/IDs/revisions/history available at this milestone, preferences/drafts/resume and all original snapshots.
- Capture DB snapshot/manifest and acquire asset leases while the writer/GC barrier is held; live WAL main-file copy is not accepted.
- G2 portable reimport verifies its declared subset and reports omitted/remapped/unsupported records; content-only is never full recovery.
- Missing linked snapshots make backup explicitly incomplete; corrupt/unsupported archives leave live workspace unchanged; agent secrets excluded.
- M4's new attempt/schedule records have a versioned export/backup adapter contract and require integration qualification before release.

Required tests
T03/T04 G1/G2 round trips, active-WAL/assets/GC races, corruption/version/path/bomb checks, failed-restore rollback, missing linked files, later-schema compatibility fixtures.

Significant risks
False completeness, inconsistent DB/assets snapshots, loss during migration or restore, evolving history types.

Exclusions
No cloud backup/sync, speculative universal file format, secure-erasure guarantee, or token export.

Supporting documentation
- `docs/architecture/data-model.md#portable-packages`
- `docs/product/user-journeys.md`
- `docs/architecture/content-ingestion.md`
- `docs/quality/verification-strategy.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

### M4.1 — Deliver activity authoring, feedback, and durable study sessions

Purpose
Support meaningful practice across recall, reasoning, production, and procedure.

Scope
Validated built-in recall/cloze, single-choice, constructed-response/problem and procedure families; authoring/import forms, rubric/solution/hints, learner-paced sessions and frozen assessment revisions.

Deliverables
Activity registry/migrations/renderers, exactly-once attempt/evidence services, draft/session resumption, suitable feedback and four-subject practice fixtures.

Dependencies
M2 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria

* Every V1 family can be created and completed without AI; unknown future types are preserved inertly rather than executed.
* Store raw response, assessment outcome, self-rating and help separately; cooking self-report and choice recognition never become verified mastery.
* Attempt/evidence/session advance commit atomically and idempotently; kill/restart loses no acknowledged attempt and recovers drafts.
* Editing content retains historical prompt/answer/rubric revisions and flags changed relevance.
* Extend ownership exports/backups with versioned activity/attempt records; never cascade-delete history.

Required tests
T01/T05/T08/T10 registry/schema, feedback/hints, replay/crash/draft/resume, safe rendering and four-subject flows; export adapter integration with M3.2 when available.

Significant risks
Semantic self-grading ambiguity, stale evidence, premature mastery claims, editor/interaction accessibility.

Exclusions
No code execution, symbolic/AI semantic grader, psychometric testing or executable activity plugins.

Supporting documentation

* `docs/product/learning-model.md`
* `docs/architecture/data-model.md`
* `docs/research/learning-science.md`
* `docs/quality/verification-strategy.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

#### Evidence refinement — 2026-10-27

Keep the four activity families; implement reusable presets/phases for focused self-explanation, optional prepared teach-back, worked-example completion/fading, clearly labeled error repair and changed-context application. Support instruction-first and bounded attempt→instruction→retry; no universally superior question format or compulsory hint delay.

Additional acceptance/tests:

* Store response format, target outcome, reference/rubric, phase and reveal/help state; resume them without losing or duplicating attempts.
* Choice items may require inference; record supplied cues and avoid inferring uncued production. Keep observed help, learner-declared external help and unknown assistance distinct.
* Deliberately flawed examples require a label and expected correction/reference with review provenance. Schema validity is not factual verification.
* Exercise these variants with shared English/Python/math/cooking contracts; no code execution or subject-specific engine.

Evidence/design: `docs/research/learning-activities.md` (A1–A7); `docs/research/consensus-review-audit.md`; `docs/product/learning-model.md`. These documents are local/uncommitted; research findings are not completed product tests.

### M4.2 — Add transparent recall scheduling and scoped assessments

Purpose
Support durable return to eligible memory work while respecting other learning outcomes.

Scope
Pinned fsrs-rs RecallScheduler adapter/default parameters, review events/state, due selection/workload controls, separate fixed/manual practical reminders, topic/subject assessment-session mode.

Deliverables
Versioned scheduler/event contracts, deterministic golden cases, user-visible scheduling policies, assessment summaries and history export support.

Dependencies
M4.1 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria

* FSRS applies only to eligible repeatable recall; procedure/problem rubric scores never feed a memory-stability model.
* Ratings, due timestamps, algorithm/parameter version and schedule generation persist with raw review history.
* Timezone/study-day/DST/clock rollback/long-gap/overdue/parameter-change cases are specified and deterministic.
* Budget changes do not delete/rewrite history; major content changes use explicit new generation/replacement decisions.
* Assessments freeze scope/revisions/feedback policy and show outcome-specific coverage/unknowns, not one mastery percentage.

Required tests
T05 scheduler fixtures/property invariants, atomic event/state replay, no-history defaults, time controls, assessment modes and export/recovery round trips.

Significant risks
Overgeneralizing memory models; wrong time semantics; transitive dependency/toolchain cost; sparse-history fitting.

Exclusions
No optimizer requirement, experimental FSRS cost/ADR, universal optimal schedule claim, or adaptive competence engine.

Supporting documentation

* `docs/product/learning-model.md`
* `docs/research/learning-science.md`
* `docs/research/existing-applications.md`
* `docs/architecture/data-model.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

#### Evidence refinement — 2026-10-27

Preserve the existing recall-only FSRS boundary. Acquisition, spaced maintenance and return after non-use are different contexts; offer appropriate instruction/refresher practice rather than treating every long gap as a stream of failures. Procedural decay averages are not individual memory parameters.

Additional acceptance/tests:

* Keep recall accuracy, assisted practice, independent checks and practical self-report distinct in assessments/exports; unknown assistance never defaults to unaided.
* Deliberately selected contrast sets/blocked or mixed practice remain configurable; no arbitrary cross-subject shuffle or universal interval/sequence claim.
* Verify content/schedule version, aid/reveal policy and unchanged historical evidence across postponed sessions and refresher attempts.

Evidence/design: `docs/research/learning-activities.md` (A8 and baseline S1–S22); `docs/product/learning-model.md`; `docs/quality/verification-strategy.md`. No optimizer, new empirical experiment or universal competence model is added.

### M5 — Complete honest progress, personalization, and session continuity

Purpose
Make a growing collection understandable and comfortable for sustained independent use.

Scope
Per-outcome progress/history/mistakes/uncertainty, source/context links, Resume/Due start actions, saved navigation/views, workspace/subject/session preference inheritance, accessible themes/text/shortcuts/reset.

Deliverables
Progress projections with inspectable evidence, versioned settings/export, keyboard-first navigation and formative usability findings/fixes.

Dependencies
M3.2, M4.2 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria

* Users distinguish recall/recognition/problem/procedural evidence and stale/unassessed/self-reported states.
* Returning users start Resume/Due in one action; interruption restores meaningful reading/session/filter context.
* Journey F customization/reset cannot destroy data or hide essential actions/focus; detect shortcut/contrast conflicts.
* Keyboard/Orca can navigate large results beyond the first viewport/page with sensible focus/count/selection during updates.
* Offline journeys A/B/D/E/F and native/portable history recovery pass; usability observations are not learning-efficacy claims.

Required tests
T06/T10 projection/inheritance/component checks, resumption/settings/export integration, keyboard/Orca large fixture, small formative task sessions with limitations recorded.

Significant risks
Dashboard false precision, overwhelming configuration, focus virtualization regressions, target percentages misrepresented as results.

Exclusions
No streak/reward economy, learning-style profiling, platform expansion, or verified competence inference.

Supporting documentation

* `docs/product/customization.md`
* `docs/product/product-spec.md`
* `docs/research/educational-ux.md`
* `docs/quality/verification-strategy.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

#### Evidence refinement — 2026-10-27

Progress must explain the outcome and conditions, not collapse assistance, motivation and independent knowledge into one score.

Additional acceptance/tests:

* Show app-observed, learner-declared and unknown assistance/reveal states where they affect interpretation. Unknown external assistance is not labeled unaided.
* Keep phase/practice versus independent-check context, self-rating/confidence, item correctness and practical observations separately inspectable.
* Do not treat frequent hints as a causal diagnosis of low ability or punish help use; preserve accessible support and error recovery.
* Neither higher AI-assisted practice scores nor a motivational improvement is presented as delayed/transfer mastery.

Evidence/design: `docs/research/learning-activities.md` (A7–A14); `docs/research/consensus-review-audit.md`. Existing milestone/dependencies and the scope of personalization remain unchanged.

### M6.1 — Ship opt-in scoped MCP reads over local stdio

Purpose
Let external hosts inspect authorized learning material through stable services instead of GUI/DB automation.

Scope
Pinned official rmcp protocol adapter, packaged stdio bridge/private Unix IPC broker, explicit pairing/grants/revocation, modern2026 and supported legacy compatibility, bounded capabilities/structure/search/resource/progress reads.

Deliverables
Versioned schemas/tool documentation, connection/grant UI, no-listener transport, conformance/client fixtures and audit/redacted diagnostics.

Dependencies
M3.2, M5 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Use the same application services as UI; bridge has no direct DB/filesystem authority and app-closed returns actionable failure without daemon.
- Authorize every nested reference/snippet/chunk; progress/raw-response access requires a separate explicit grant; hidden IDs/counts do not leak.
- Self-reported client identity is not authorization; private IPC/grants fail closed on revocation, process loss or stale socket.
- Support named modern/legacy host cases through the pinned SDK; unknown revisions fail clearly, stdout remains protocol-only.
- No HTTP endpoint, network fetch, provider subscription, host credential or implicit content execution is introduced; disclose host read/export trust limit.

Required tests
T07/T08 protocol conformance and real-client smoke, scope/ID/resource denial, grant theft/revoke races, app shutdown, limits/cancellation, no-listener/offline checks.

Significant risks
Protocol/SDK churn, same-user OS threat limits, external-host exfiltration after authorized disclosure, token/log leakage.

Exclusions
No write/apply tools, raw SQL/path/URL tools, HTTP/OAuth service, sampling/tasks/apps extension or AI-provider SDK.

Supporting documentation
- `docs/architecture/ai-mcp-integration.md`
- `docs/architecture/technical-design.md`
- `docs/product/user-journeys.md`
- `docs/quality/verification-strategy.md`
- `docs/architecture/mcp-contracts.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

### M6.2 — Deliver rich MCP draft authoring, previews, and resumable jobs

Purpose
Give external agents substantial freedom to build and iteratively improve complete learning structures without a click per draft edit.

Scope
Persistent draft overlays, composable typed subject/goal/collection/unit/path/document/activity/link authoring, bounded structured import uploads, schema discovery, effective-draft reads, validation/learner previews, chunked batches and resumable jobs.

Deliverables
Versioned agent contracts/schemas/examples, durable drafts/temp-ID maps, structured error/recovery model, quality/attribution checks and substantial multi-subject course-authoring fixtures.

Dependencies
M6.1 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria

* Agent builds and revises a multi-module course with citations/prerequisites/shared resources and several activity types using ordinary subject-agnostic services.
* Draft create/read/edit/discard, validation and inert previews require no manual click per operation; restart resumes durable IDs/revisions/jobs.
* Large courses use bounded chunks/batches and dependency-closed selections; no hidden partial live apply or unbounded allocation.
* Validation checks schemas/references/cycles/scope/attribution/accessibility and returns repairable field errors, without claiming factual truth.
* AI origin/unverified state and preserved originals remain visible; no raw SQL/path/URL/execution or learner-history mutation.

Required tests
T04/T07/T08 contract/registry/draft revision, temp-ID/dependency, chunk/hash/budget, cancellation/resume, malformed input, source scope, preview inertness and four-subject authoring workflows.

Significant risks
Shallow tool surface requiring guessing, brittle batch/error handling, losing draft work, misleading content-validation claims.

Exclusions
No live application bypass, built-in model/provider, semantic truth oracle, raw database/filesystem or learner grade writes.

Supporting documentation

* `docs/architecture/ai-mcp-integration.md`
* `docs/architecture/content-ingestion.md`
* `docs/product/user-journeys.md`
* `docs/quality/verification-strategy.md`
* `docs/architecture/mcp-contracts.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

#### Evidence refinement — 2026-10-27

Keep rich agent authoring freedom. Typed draft operations/schemas must express reusable instructional phases and variants (example/completion, explanation/teach-back, error repair, independent application), with objective, response mode, citations, expected correction/rubric and declared help policy.

Additional acceptance/tests:

* Validate missing references/roles/correction fields and clearly label intentionally erroneous examples. Validation never certifies educational truth; AI/unverified provenance survives.
* Preview and resume both example-first and bounded attempt-first plans through ordinary shared services, without a new subject-specific engine.
* Pedagogical answer/help policy does not change the UI-delegated authorization model or require a click for every safe draft edit. Learning-assistance RCTs do not justify weaker MCP authoring grants.
* Agents still cannot write learner attempts/grades; external assistance remains declared/unknown unless actually observed by the application.

Evidence/design: `docs/research/learning-activities.md`; `docs/research/consensus-review-audit.md`; `docs/architecture/mcp-contracts.md`. Keep the existing M6.3 scope/impact/history protections.

### M6.3 — Enforce reviewed and delegated autonomous MCP application

Purpose
Allow useful reversible agent action under explicit user authority while protecting sensitive changes and learning history.

Scope
UI-created specific approval and scoped time/budget/operation autonomy policies, exact validated hashes, human-edit ownership protection, transactional grant/revision/budget checks, apply receipts/audit, safe compensating undo and escalation.

Deliverables
Understandable authority-mode UI, policy engine shared by all mutation paths, default-review and delegated-authoring workflows, operation receipts and race/fault fixtures.

Dependencies
M6.2 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Default review applies only exact UI-approved selections; the agent cannot submit an approval flag or create/widen grants.
- Explicit delegation automatically applies safe additions/edits/organization in the chosen scope without a click per operation; sensitive or human-edited targets escalate.
- Recheck scope, policy version, expiry, revocation, revision/hash and remaining budget inside commit; parallel operations cannot double-spend authority.
- A resource/unit shared between granted and denied areas cannot change denied areas through a granted placement: authorize effective impact or explicit logical-object authority, safely fork or escalate without hidden-metadata leaks.
- Agent-owned assessed content is not freely overwritable: protected semantic changes require review or an explicit prior relevance policy; immutable attempts/grades remain intact and schedule-generation consequences follow that policy.
- Stale edits, invalid dependencies/cycles and revoked/oversized changes roll back the selected atomic group; lost-response retries recover the same receipt.
- Undo respects later edits/history; no originals, attempts/grades or out-of-scope records are silently changed; AI content stays identified/unverified.
- Journey C tests rich draft authoring, both authority modes, interruption, PDF fallback/original access, then ordinary offline study after disconnect.

Required tests
T07/T08 forged approval/policy, nested scope leaks, protected human edits, sensitive-operation escalation, parallel budget/revoke/expiry races, stale/hash conflicts, commit cancellation/idempotency/undo conflicts and source/history preservation. Include shared-resource A/B impact/fork/escalation and non-leaking denial tests. Include agent-created content revised after learning evidence, protected fields and explicit relevance-policy consequences.

Significant risks
Authority mode misunderstanding, approval fatigue, autonomous scope creep, race conditions and convincing incorrect educational output.

Exclusions
No history/grade/grant mutation tools, deletion/purge authority, unrestricted localhost API, autonomous built-in AI or factual-correctness guarantee.

Supporting documentation
- `docs/architecture/mcp-contracts.md`
- `docs/architecture/ai-mcp-integration.md`
- `docs/product/user-journeys.md`
- `docs/quality/verification-strategy.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.

### M7 — Qualify and prepare the Fedora Foundation V1 release

Purpose
Demonstrate the complete product, recovery, and safety claims in a supported packaged environment.

Scope
Reproducible RPM/runtime dependency/signing plan, upgrades/migrations/recovery/crash/security/scale/accessibility/offline qualification, reviewed original English/Python packs, math/cooking architecture fixtures, license/SBOM/notices and support docs.

Deliverables
Release evidence matrix T01–T10/P01–P14/A–G, measured budgets/dispositions, known limits, reviewed packs/data attribution, installation/recovery documentation and release candidate.

Dependencies
M1.2, M2, M3.1, M3.2, M4.2, M5, M6.3 See docs/planning/roadmap.md; prerequisite completion must be verified in source, not assumed from planning.

Acceptance criteria
- Fresh Fedora RPM completes all core journeys offline; actual packaged keyboard/Orca/IME flows and large-result focus qualification pass.
- G1 full recovery verifies all source snapshots/IDs/revisions/history/drafts/settings; G2 preserves declared subset; corruption and failed upgrade leave prior data recoverable.
- Four subject types use shared architecture; only English/Python are user-facing example packs with verified independent content/media rights.
- Hard integrity/security/AI-scope/MCP/accessibility gates pass; conditional PDF decision is explicit and tested, never silently claimed.
- Record measured performance/resource results, vulnerability/dependency/license inventory and unresolved limits; no scientific learning percentage claimed.
- Publication/distribution/signing authority is separately obtained when requested; preparing a release candidate is not permission to publish.

Required tests
Full release suite/integration/property/boundary corpus, clean install/update/reinstall/crash/restore, SDK/client smoke, manual accessibility/usability qualification and documented large fixture profiling.

Significant risks
Unqualified distro/runtime variants, incomplete dependency notices, misleading recovery/effectiveness claims, release-key custody.

Exclusions
No Windows/macOS/mobile/sync/accounts/cloud, other unqualified package promises, original efficacy experiment, or automatic publication.

Supporting documentation
- `docs/product/foundation-v1.md`
- `docs/planning/roadmap.md`
- `docs/quality/verification-strategy.md`
- `docs/architecture/third-party-reuse.md`

These Foundation documents are currently local/uncommitted. This is a future implementation ticket in Memoquill only; creating it does not start implementation or authorize commits/push/publication.
