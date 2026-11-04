# Lesson composition contract — KER-117

Status: concrete architecture proposal, 2026-11-04; awaiting user agreement.
This document specifies desired behavior, not implemented APIs, tables or acceptance.
Application development and GUI qualification remain paused. Normative words below
apply to the proposed contract only. Existing IDs and codecs remain authoritative.

## Intent, constraints and alternatives

Learners manually compose useful lessons offline; a future MCP client uses the same
semantic services. Replacing a screen must not change content, evaluation or history.
Subject independence, preserved sources, accessibility and explicit learner choice
are constraints. Neither an AI author nor a cloud account is required.

| Approach | Benefit | Cost / disposition |
| --- | --- | --- |
| Existing Unit plus owned composition revisions and small trusted renderer | Preserves identities; smallest vocabulary with explicit safety and history semantics | Own renderer/authoring controls; recommended baseline |
| Same domain model with a json-render adapter | Reuses catalog/registry/rendering patterns | Translation, dependency and renderer-specific condition semantics; optional later adapter |
| New Lesson aggregate containing copies of content and outcomes | Self-contained UI object | Duplicates Unit ownership and breaks shared revision/history semantics; rejected |

Generated React/JS/HTML is outside the data contract. No dependency or executable
extension is selected. A renderer spike would require separate authorization.

## Identity, ownership and reference closure

**Lesson is the user-facing editable projection of an existing learning Unit.**
`lessonId = unitId`; do not create a second logical Lesson identity. Composition is
a new revisioned record owned by that Unit, not by a tab or collection placement.
Each Unit has at most one active composition head in contract v1. Variants are
drafts; separately reusable variants later require their own explicit design.

Tabs are views: `(workspaceId, unitId, viewInstanceId)`. Opening the same lesson
twice shares the authored composition but has independent focus/selection/scroll.
Moving/renaming a collection entry preserves Unit, composition and history IDs.
An existing resource opened alone remains a resource view. Creating a lesson from
it explicitly creates a Unit and a pinned reference, without copying its bytes.
Paths order Unit references; they do not own compositions. Units without a saved
composition retain their existing usable representation until explicit conversion.

| Record | Owner / revision | References and mutation rule |
| --- | --- | --- |
| Canonical resource | Existing logical resource; immutable document revisions | Stable document block IDs remain distinct from component node IDs |
| Activity definition | Existing logical activity; immutable revisions | Prompt, answers, rubric, hints, evaluation and target outcomes belong here |
| Composition | Unit; immutable composition revisions and mutable checked head | References exact resource/activity revisions, bindings and trusted type versions |
| Source binding | Unit; immutable binding revision | Existing source/provenance ID, asset hash, extraction revision, bibliography, locators, selected ranges and coverage |
| Session/attempt | Learner profile; session snapshot and append-only evidence | Pins Unit/composition/activity/resource/binding revisions; never owned by a component |
| Draft/proposal/receipt | Actor and authorized Unit scope | Durable authoring overlay and operation identity; contains no learner outcome authority |

A composition revision contains `contractVersion`, `unitId`, `revisionId`,
`parentRevisionId`, `registryContractVersion`, `rootNodeId`, `nodes`, `bindings`,
`sourceBindingRefs`, `appearance`, `authoringProvenance` and its canonical hash.
Revision IDs are server-issued immutable identities; parent is the accepted head.
The content hash detects exact payload identity, not permission or authorship.

Bindings are named local references: `{bindingId, targetKind, targetId,
revisionId, locator?}`. Every dependency is pinned; no stored `latest` reference.
Moving the target head offers an explicit update proposal. Source bindings describe
actual `extractionCoverage`, warnings and `usedRanges`; agent read receipts record
actual ranges read separately. Selected pages do not imply an entire book was read.
Original bytes and bibliography are not copied per tab. Unavailable sources retain
their identity and an honest unavailable/partial/original-only status.

Reference resolution validates workspace, existence, kind, revision, locator and
read authority. A citation is not a grant. Editing a shared resource requires the
existing logical-object impact rule in [MCP contracts](mcp-contracts.md); lesson
write alone permits rebinding/forking into its scope, not rewriting shared targets.
Forking creates new IDs and provenance, never copies mastery or attempts.

## Components, activities and layout

Component node shape: `{nodeId, typeId, typeVersion, props, slots, on, visible?}`.
`slots` maps named slots to ordered child IDs. Nodes form one rooted tree: every
node is reachable, has one parent except root, no cycle, no duplicate child.
Cross-links to content use bindings, never additional containment parents. Reorder
and reparent retain node IDs. A new node receives a new ID; cloning never aliases it.

The registry is shipped and reviewed with the application. Each descriptor defines
type/version, closed typed prop schema/defaults, slots and child cardinality,
event payloads, allowed named actions, state scope/schema, required capabilities,
accessibility contract, migration support and inert fallback. Components cannot
declare their own grants. Registry/schema discovery is available offline to manual
controls and authorized agents, including unsupported-type diagnostics.

| Initial type | Typed configuration / slots | Events and semantics |
| --- | --- | --- |
| `stack@1` | gap enum, heading level; ordered `body` | Semantic reading order equals child order |
| `columns@1` | ratio enum; `primary`, `secondary` | Linearizes primary then secondary; responsive collapse keeps this order |
| `section@1` | accessible title, collapsible boolean; `body` | `toggle(bool)` affects view state only; hidden content is not secret |
| `content@1` | resource binding and optional stable block range | Passive rendering of canonical safe document; no content duplicated in props |
| `source@1` | source-binding revision and locator | `openSource` via managed resolver; no arbitrary path/URL fetch |
| `activity@1` | activity binding; presentation enum | Response, hint, reveal, submit events routed to activity controller; policy comes from activity revision |
| `button@1` | label, style enum, declared action binding | Keyboard activation and accessible name; no handler text |
| `status@1` | declared read-only view field, label | Completion/status with explicit meaning, never a claim of universal mastery |

Initial activity families reuse [the learning model](../product/learning-model.md):
recall/cloze, single-choice, constructed response/problem, and procedure/practical
checklist. Each type has a separate versioned payload and evaluation descriptor.
Support exact-key comparison only where authored and explicit; constructed work
uses self/rubric assessment with unknown outcome available. No implicit code runner,
semantic grader, symbolic solver, or verification of physical performance.
Worked examples are resources; solve-and-compare links an activity to them.
New teaching methods can combine these without making every method a runtime type.

Each activity node is an independent occurrence, even if two nodes bind the same
activity revision. Session items receive distinct IDs and pin `(compositionRevisionId,
nodeId, activityRevisionId, occurrenceOrdinal)`. Response/hint/reveal state belongs
to that item, never just the activity ID. Repeated sessions create new items;
preview uses the same occurrence rule in its isolated store. Schedule aggregation
may still reference the logical activity, but cannot conflate item attempts.

Appearance uses bounded named tokens (spacing, contrast-safe theme, typography,
width and emphasis enums), not CSS, selectors, DOM attributes or URLs. Focus and
keyboard alternatives must exist for move/nest/configure; no drag-only operations.
Headings/labels, error announcements, focus restoration and meaningful sequence
are registry obligations, later qualified on Fedora/Orca, not established here.

## Actions, conditions and state

An action descriptor supplies `actionId/version`, closed input/result schemas,
target kinds, required capabilities, valid modes, side-effect class, idempotency
policy and error vocabulary. Bindings map typed constants or declared event fields
to inputs. No expression interpolation, prototype keys, dynamic action names,
scripts, recursion, timers or network/filesystem actions are allowed in payloads.

| Named runtime action | Authority and state |
| --- | --- |
| `view.toggleSection`, `view.focusNode` | Same lesson/view, ephemeral UI state |
| `source.open` | Authorized source binding and locator through managed resolver |
| `response.update` | Typed response draft for current session item, checked save sequence |
| `study.hint`, `study.reveal` | Controller checks pinned phase/policy and records actual assistance |
| `study.submit` | Service validates attempt; atomic evidence/session advance with receipt |
| `study.next` | Session policy controls advance; no fabricated correctness/completion |

M2 defines these descriptors and preview simulations. Durable study/evaluation
implementation belongs to M4/KER-102; unavailable runtime capability is explicit,
never a successful no-op. Components emit events, controllers request actions, and
services enforce authority even if a button is hidden or the UI is replaced.
One user event invokes at most one action; compound transactional authoring changes
use the bounded operation batch below. Service results cannot automatically emit
new action chains. Input-change events may save drafts, never submit attempts.

Conditions have a closed AST: `literal(bool)`, `eq(field, scalar)`,
`compare(field, lt|lte|gt|gte, finiteNumber)`, `not(expr)`, `all(expr[])`,
`any(expr[])`. `field` is a registry-declared local view field ID, not a JSON path.
Types are strict: no coercion; missing or invalid field disables the condition and
shows a diagnostic. No joins, regex, arithmetic, loops/repeats or history queries.
Conditions influence visibility/enabled state only. Policy protects hints, answers
and unauthorized data at the service/controller boundary; they must not enter a
restricted view model merely because a node is hidden.

| State | Key / lifecycle | Export and version behavior |
| --- | --- | --- |
| View | view instance + node; focus, selection, expansion | Ephemeral; discarded safely, no history |
| Reading preferences | profile + resource/revision | Existing preference contract; not composition content |
| Author draft | actor + Unit + draft revision | Durable; explicit content proposal export, no response data |
| Preview | preview ID + pinned draft revision | Isolated disposable store; cannot call real attempt submission |
| Real response/session | profile + session/item + snapshot | Recoverable typed drafts; M4 atomic submission, not component-local storage |
| Learning history | profile + attempt/evidence IDs | Append-only, optional explicit history export |

M8 scores/rules stay absent from v1 execution. Future lesson-local gamification is
default-off and cannot mutate independent evidence or cross-lesson state.

## One semantic authoring service

Manual controls and later MCP adapters use `Discover`, `CreateDraft`, `EditDraft`,
`Validate`, `Preview`, `Apply`, `GetReceipt`, `DiscardDraft`. Names describe an
application port, not new transport endpoints. Rust services own validation,
transaction/recovery and authorization. KER-118 owns UI/controller/native/test
adapters and guest identity; KER-119 persistence; M6 live scoped stdio MCP.
Current `RequestContext` is a trusted-UI flag in source, not an existing agent grant
system. Native/broker code must construct future actor/grant context; payloads
cannot claim trust. Actor/profile identity is separate from grant authority.

Mutation envelope: `{contractVersion, workspaceId, unitId, requestId,
expectedDraftRevision?, expectedHeadRevision?, operations}`. Context is supplied
by the trusted caller boundary. Create requires Unit write, or explicit create-Unit
authority in its destination; it must not be hidden inside a layout-only mutation.

Closed operations: `insertNode(parent, slot, index, node)`, `removeNode(id)`,
`moveNode(id, parent, slot, index)`, `configureNode(id, typedProps)`,
`bindReference(binding)`, `setAppearance(tokens)`, and explicitly typed
`createResourceRevision` / `createActivityRevision` with their own bases and grants.
Creation initializes a valid `stack` root. No raw JSON/database patch or script.
`configureInteraction(id, on, visible)` replaces a node's typed event bindings and
condition with the same node ID, using registry/event/action validation and normal
draft revision checks. Omitted `visible` means true; `on` is a complete replacement
map (empty clears it), not an arbitrary merge. Initial insert validates these fields
by the same rules. It cannot change real session policy or an active snapshot.
Removing a node removes its subtree from this composition only. It does not delete
resource/activity definitions, sources or attempts. Root removal is rejected.
Moves use indices in the target slot after removal; validate the final tree.
Undo creates an inverse draft edit/new head revision and rechecks current bases;
it never erases committed revisions or concurrent edits.

1. Edit uses expected draft revision; normalize only documented defaults, reject
   unknown fields/types on new authoring. Return new revision and affected IDs.
2. Validate dependency closure, all bases, schema, authority/shared impact, budgets,
   accessibility requirements and unknown-node restrictions. Return structured
   diagnostics `(code, node/binding/operation, field, repair)` with no hidden data.
3. Preview uses exact validated draft revision, registry version and dependency
   revisions. Return diff, warnings and impact summary/hash; simulation never writes
   learner evidence. A changed draft invalidates the preview.
4. Apply requires exact preview/proposal hash, current bases and either explicit
   review or matching existing delegation policy. Manual Save is the user's explicit
   application action; an agent cannot approve itself. Recheck grants/revocation,
   dependency closure and budgets at commit, not only preview time.
5. Single-writer transaction commits selected dependency-closed revisions, head,
   audit, budget consumption and receipt together. Stage assets durably first using
   the existing asset protocol. Any invalid operation rolls back the whole batch.

Receipt includes request/operation ID, actor, proposal hash, old/new heads and
created revision IDs, temporary-to-stable ID map, warnings and committed status.
Idempotency is keyed by workspace/principal/request ID plus canonical payload hash;
same request returns the original receipt, changed payload gives `idempotency_conflict`.
Revoked callers receive no receipt details beyond their current read scope.
Lost replies use `GetReceipt`; cancellation after commit returns committed outcome.
Conflicts preserve draft; refresh/rebase is explicit, no silent last-write-wins.
Errors include `revision_conflict`, `permission_denied`, `unsupported_version`,
`invalid_reference`, `invalid_tree`, `budget_exceeded`, `needs_review`,
`capability_unavailable`, `recovery_required`; diagnostics do not expose hidden refs.

## Versioning, migration and recovery

Keep database schema version, composition contract version, type versions,
resource/activity revisions and registry implementation version distinct.
Implementation patch changes cannot reinterpret stored evaluation semantics.
Breaking serialized semantics require a new type/contract version and trusted
migrator. Runtime/evaluator versions used by real attempts are retained in evidence.

Migration is deterministic, bounded, application-shipped and produces a proposed
new immutable revision plus mapping/report; originals are preserved. It cannot
run code from packages, fetch dependencies or silently change activity evaluation.
Before activation validate migrated references, budgets, accessibility and hashes.
Failure leaves original head usable or explicitly read-only, never partly migrated.
Database upgrades retain backup/transaction rules from [data model](data-model.md).

Unknown types/versions from imported or newer compositions remain bounded opaque
data with inert placeholders and export fidelity. Never execute or silently drop
them. Known siblings may render as a degraded read-only preview; activating a new
head or starting study with unknown required nodes is blocked. Authoring repair
must explicitly replace/remove them with impact preview. No editing inside opaque
payloads. Oversized/invalid imports are quarantined as original packages outside
the active composition store; preservation is not permission for unbounded decode.

Existing Units acquire compositions lazily through explicit conversion: retain
Unit/resource/block/activity IDs, generate only new composition/node/binding IDs,
pin selected revisions, show diff, then apply. No bulk rewrite or SQL in KER-117.
Reopen/export/restore must include full pinned dependency closure and original
unknown payloads; content-only export excludes attempts/drafts/preferences/grants.
Private originals require explicit inclusion; exclusions/completeness are reported.

Starting a real session pins its full snapshot and evaluator versions. Publishing
a changed lesson does not modify an active session. The learner may finish the
old snapshot or explicitly end it and start a new session; no in-place mid-item
migration. Removed nodes/activities remain resolvable for history. Regrading creates
explicit superseding evidence; composition changes never reset schedule/history.
AI authoring provenance and observed/declared/unknown learner assistance are separate.

## Proposed hard limits (engineering defaults, not learning findings)

| Boundary | v1 ceiling / failure behavior |
| --- | --- |
| Composition serialized JSON | 1 MiB UTF-8, no inline asset bytes |
| Tree | 256 nodes, depth 12, 64 children per slot |
| Props / bindings | 16 KiB per node; 256 bindings and 64 source bindings |
| Conditions | 32 AST nodes, depth 4 per condition; 4096 evaluations per event |
| Action bindings | 4 per node, one action per event; input 16 KiB |
| Edit/apply | 64 operations and 2 MiB envelope; dependency closure at most 512 revisions |
| Drafts | 20 open drafts per actor/Unit, 32 MiB total persisted payload per actor/Unit |
| Component view state | 64 KiB per view; response limits owned by activity schema |

Validate byte/depth limits before recursive decoding/allocation; finite JSON numbers,
closed schemas and bounded strings/arrays everywhere. Limits are cumulative across
the request, not bypassable through batching. Oversized work must split into explicit
independent proposals; atomic intent cannot silently become partial application.
Report budget diagnostics without truncating authored data. Local desktop qualification
may lower these defaults before release with a versioned compatibility report; raising
them requires resource/security review. No performance guarantee is claimed here.

## Four subject traces and acceptance scenarios

| Subject / goal | Composition and evaluation | Continuity / honest evidence |
| --- | --- | --- |
| Language: unaided word recall | Source excerpt + recall activity + optional reveal | Pinned prompt; reveal records assistance; recall does not establish conversational ability |
| Python: explain a bug | Code resource + constructed response + worked example/rubric | Code stays passive; store explanation and self/rubric result; no execution or fabricated correctness |
| Quantum physics: derive a result | Source equation locator + staged problem/hint + comparison | Preserve source range and rubric versions; changing layout does not regrade derivation or reset response |
| Cooking: perform a procedure | Source recipe + checklist + observations/reflection | Self-reported steps distinct from independent practical competence; offline artifacts and history retained |

All four use the same Unit/composition/binding/action schemas, with subject labels
as data. They are architecture traces, not product trials or effectiveness evidence.
Custom safe methods remain possible without claiming scientific validation.

Implementation acceptance must cover manual/agent equivalent operations and receipts;
same-lesson concurrent tabs; shared resource denial/fork without disclosure;
retry-after-lost-reply and changed-payload rejection; revoked grant at apply;
cycle/depth/byte/condition limits; unknown round-trip/quarantine/migration failure;
preview isolation; offline save/reopen/export/restore dependency closure; pinned
ongoing sessions across edits; retained anchors/history; keyboard reordering,
reading order/focus and screen replacement without domain/storage changes.

First slice after agreement: one existing Unit, stack/content/activity nodes,
insert/configure/move/remove, isolated preview, exact apply/reopen and conflict
recovery through shared services. Study is simulated until M4. Columns, source
reader capabilities and remaining vocabulary follow within their owning milestones.
Do not call KER-120 or integrated M2 complete from this slice alone.

## Agreement and implementation discretion

User agreement is still needed on Unit ownership, immutable pinned revisions,
owned-renderer baseline, restricted vocabulary/limits and first slice. Builder may
choose internal module/table names, view controls and indexing strategy consistent
with these contracts. Dependency selection, executable extensions, real auth,
gamification and study implementation are outside this checkpoint. KER-117 remains
In Progress until architectural agreement; only then prepare an implementation plan.

Evidence: [existing architecture research](../research/composable-learning-architecture.md),
[screening and source contracts](../research/composable-learning-screening.md),
and [this checkpoint's narrow evidence ledger](../research/lesson-contract-evidence.md).
