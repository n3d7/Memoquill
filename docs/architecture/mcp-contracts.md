# MCP agent contracts and delegated autonomy

This is the detailed Foundation V1 design contract, not an implemented API. [MCP architecture](ai-mcp-integration.md) owns transport/trust boundaries; this document owns the agent-facing operating model. The user's clarified requirement is **substantial agent freedom and capability**, with safe application-level authority rather than GUI automation or direct database access.

## Design goals

An external agent must be able to understand an unfamiliar workspace, read originals and existing learning structures, construct a substantial course, inspect its work, fix validation errors, collaborate with human edits, and resume after interruption. It must not need a human click for every draft edit or every action within an explicit reversible-operation delegation.

The interface is subject-agnostic and composable: identical operations build English vocabulary/production, Python concepts/non-executed exercises, mathematical prerequisites/problems, and cooking procedures. No provider-specific endpoint, “AI subject” schema, hidden generic SQL/shell tool, or one-shot card generator substitutes for this surface.

## Authority modes

| Mode | Agent freedom | Live-data boundary |
| --- | --- | --- |
| Scoped inspection | Discover authorized structures/schemas, query/search, read original/normalized material and permitted progress. | No write; original/raw-response access can be narrower than ordinary content access. |
| Draft authoring (default write mode) | Create/edit/reorder/link/delete its own uncommitted draft objects, stage structured imports, build paths and activities, inspect effective content and previews, validate repeatedly, save/resume jobs. | Persistent draft overlay only. Human can inspect/reject; original/live records untouched. |
| Delegated reversible authoring | Automatically apply allowed additions/edits/organization in an explicitly selected scope, within time/object/byte/operation budgets and ownership rules. | UI creates the policy; every transaction rechecks it. AI provenance/unverified state remains. No per-operation human click when all conditions pass. |
| Specific review | Propose sensitive edits, replacements/merges, overwrite of human-edited material, scope expansions or other nondelegated operations. | Exact UI-reviewed hash/base revisions required. Agent cannot approve, widen scope or reset budgets. |

Autonomy is a user choice, not a scientific recommendation. Defaults remain understandable: “Inspect”, “Prepare changes”, and “Allow reversible authoring in this area”. Advanced operation/size/duration limits are discoverable rather than mandatory setup.

A policy identifies workspace, subject/collection/resource scope, allowed operation classes, allowed relationship targets, expiry, operation/byte/object budgets, ownership rules, and policy version. New object scope is derived from its target placement; a foreign reference must separately be readable/linkable. A display label or model claim is never authority. Revocation and budget consumption share the apply transaction, preventing parallel requests from spending the same budget twice.

### Delegable vs sensitive operations

- **Delegable:** create subjects only if explicitly allowed; create collections/goals/units/paths/lessons/activities in the selected area; revise agent-created items not later human-edited; add citations/tags/links; reorder/move existing placements only under an explicit organization grant; archive its own newly created unreferenced draft/live additions when safe.
- **Always specific review in V1:** overwrite human-edited lesson/answer/rubric, merge/replace existing logical identities, delete source snapshots, purge/archive existing history-bearing objects, move across grant boundaries, or disclose/export data not already covered by read/original/history grants.
- **Unavailable to agents in V1:** grant/approval-policy mutation, raw database/filesystem/network/code execution, learner attempt/review/grade mutation or fabricated practical-competence evidence. Agents can propose a study plan or labeled draft feedback instead.

An agent-owned item becomes protected against autonomous overwrite after a human edit. Origin remains AI-generated even when a human later reviews it. Ownership is an authorization attribute with revision checks, not a blanket right to every descendant/reference. Conflict escalation never silently downgrades a human edit.

**Shared-object impact rule:** authorizing one placement does not authorize changes to a logical resource/unit used elsewhere. Compute the effective impact of revisions, moves, links, archives and replacements across existing placements/references. A live mutation requires either an explicit logical-object write grant whose shared effect the UI explains, or authority over every affected scope. Otherwise fork into the allowed scope with a new identity (preserving provenance, never copying mastery/history) or request specific review. Denial/escalation must not reveal hidden placement names/counts/content. Placement-only edits are distinguished from mutations of the shared target.

**Learning-evidence impact rule:** agent ownership alone is insufficient once an activity or dependent objective/material has learner evidence. Changes to protected prompt/answer/rubric/objective/procedure/content fields require specific review or a prior UI-selected bounded relevance policy (for example, create a new variant while retaining the old item, or explicitly mark prior evidence stale under defined rules). Do not trust an agent's “typo only” claim to bypass protected-field checks. Metadata/placement edits that do not change assessment meaning can remain delegable. Existing attempts/grades/review events stay immutable; any schedule-generation consequence follows the explicit user policy, never an arbitrary agent-written schedule. Human-edited protected content still requires specific review.

## Discovery and domain surface

Tool groups map to shared application services; implementations may combine narrowly related verbs into typed operation unions, but must not accept free-form executable commands. Publish machine-readable schemas, error variants, examples and capability requirements with versioned application contracts. All lists/searches paginate and filter before returning private counts/snippets.

| Family | Essential operations and useful outputs |
| --- | --- |
| System/capabilities | Contract/spec adapter versions, granted scopes/modes, expiry/budget remaining, supported activity/document types, schema/handler versions, size limits, read-only/draft/live status. Denied capabilities are described without revealing hidden subjects. |
| Workspace/subjects | List/get authorized subjects, goals, prior-knowledge notes and preferences that are safe to disclose. Create/update subject/goal in draft or delegated scope. No global learner-profile enumeration. |
| Organization | Read collections/placements/tags, units, prerequisite/related/reference edges and learning paths. Query bounded neighborhoods and inverse references; identify orphans/cycles/uncovered objectives. Typed filters, not arbitrary Cypher/SQL. |
| Content/source | Search titles/full text/tags/metadata, get outlines/blocks/revisions/source anchors, read bounded original chunks and attribution, inspect derivations and changed/missing anchors. Source bytes and generated summaries remain distinct. |
| Draft workspace | Create/list/read/edit/clone/discard own drafts; get effective overlay, changes/diff, temporary-to-stable ID mapping, base revisions and validation status. Work continues across client/app restart. |
| Structured authoring | Typed create/revise operations for documents/blocks, units/goals, resources/citations, activity payloads/solutions/rubrics/hints, paths/steps and relationships. Bulk reorder/tag/link operations avoid one tool call per trivial item. |
| Import preparation | Start staged upload/import, append bounded chunks with hashes, finalize/inspect mapped records, choose source-key mapping, inspect duplicate/conflict/fidelity reports, attach candidate sources to drafts. No arbitrary server-side path/URL. |
| Validation/preview | Schema/reference/scope/cycle checks; source attribution/anchor presence; missing answer/rubric/alt text, malformed cloze and unsupported-node warnings; inert activity/lesson/path/session view models. Structural quality checks do not verify educational truth. |
| Proposal/application | Submit a draft revision, validate selected dependency-closed operations, determine required review/delegation, apply exact validated revision under current authority, return operation receipt/ID mapping/conflicts. |
| Planning/progress | With separate permission, inspect outcome-specific summaries, mistakes/coverage/help, and optionally raw responses; propose sessions/learning paths or feedback. No writable mastery/grade/history fields. |
| Jobs/operations | Status/progress/errors/cancellation and resumable cursors for imports/validation/batches; inspect own scoped receipts/diffs and request nonconflicting compensating undo. Terminal outcomes distinguish applied, rejected, cancelled, failed and needs-review. |
| Portability | Prepare a scoped export manifest/preview under explicit content/history/original-read authority; read resulting bounded bytes when allowed. No network destination or ungranted hidden dependency inclusion. |

Resource URIs and tools share authorization and revision checks. An agent must be able to inspect a source referenced in a lesson if granted, but a citation never implicitly grants access. “List types/schemas” and “read validation errors” are first-class capabilities so agents can repair mistakes without guessing application internals.

## Common envelopes and invariants

Requests include application contract version, operation-specific payload, workspace ID, request/idempotency ID for mutations, and expected draft/entity revisions where relevant. Scope is server-derived from the authenticated grant, never trusted from a request's claimed scope. Replies include result/status, revision/snapshot identifiers, explicit continuation, warnings, affected counts, remaining budget and an operation/job ID when relevant.

- **IDs:** stable UUIDs; drafts allocate stable candidate IDs or explicit temporary-ID maps. Names are presentation, not identity. Dependencies between draft objects resolve before apply.
- **Schemas:** named/versioned document/activity types, strict allowed fields, unknown-type preservation rules and safe migrations. Unknown mutation fields are rejected rather than silently ignored.
- **Read consistency:** immutable content revisions and bounded snapshot/cursor tokens. A cursor binds query/scope/grant/snapshot; expiry or revocation fails clearly. Clients can refresh deliberately after changes rather than concatenate inconsistent pages.
- **Draft consistency:** each edit uses expected draft revision; return its new revision and effective changes. Do not silently merge concurrent agent/human edits. Draft dependency closure/reference scope is revalidated on submit/apply.
- **Idempotency:** key is principal/workspace/operation scoped. Same key+same payload returns the prior result; same key+different payload errors. Persist successful receipts across process restart for the documented retention window.
- **Batches:** ordered typed operations can reference earlier candidates. Validate a complete selection before live application; default atomic transaction. For larger plans, expose explicit independently atomic groups and their dependencies/receipts; never claim whole-course atomicity after partial commits.
- **Budgets:** distinguish request/chunk cap, draft/total job allowance, transaction operation cap and concurrency/time budget. A course larger than one request can be uploaded/edited in chunks. User-selected larger quotas remain bounded; the agent cannot raise them itself.
- **Data provenance:** original/imported/human/AI origin, citations, transformation/model metadata, reviewer and review state are separate fields. A structural validation pass never sets “factually verified”.

The app service checks scope for every nested reference, generated item destination, attachment, relationship and effective shared-object impact, including references not named in the request. It must not use global fuzzy matching that leaks another subject into a duplicate report. Search/draft preview cannot become a side channel around denied original/progress reads.

## Error/recovery contract

| Stable error class | Required agent-visible recovery information |
| --- | --- |
| `unsupported_contract` / protocol incompatibility | Supported application/schema/protocol versions; no silent lossy mutation. |
| `permission_denied` / `grant_expired` / `revoked` | Capability category and safe reauthorization route, without hidden object metadata. No endless automatic retry. |
| `approval_required` | Scoped proposal ID/hash, reason/category, UI review state; no flag that bypasses it. |
| `revision_conflict` | Authorized affected IDs/current revisions and safe refresh/rebase path; preserve draft and human edits. |
| `validation_failed` | Field/operation index, stable code, expected schema, related accessible source references; distinguish error vs warning. |
| `budget_exceeded` / `payload_too_large` | Specific remaining limit, supported chunk/group/resume method; no partial hidden apply. |
| `cursor_expired` / `snapshot_unavailable` | Resume/refresh instructions and last durable operation ID. |
| `job_cancelled` / `partial_import` / `unsupported_extraction` | Per-item terminal states, preserved originals and accepted/rejected counts; retry only uncommitted work. |
| `app_not_running` / storage failure | Actionable user step, whether any transaction committed, stable receipt if present; never retry a mutation blindly. |

Cancellation affects uncommitted work; once commit succeeds, return/recover its receipt rather than report “cancelled” as if nothing changed. Resource/error text is escaped/inert data and cannot instruct the broker to expand authority.

## End-to-end agent workflow

1. Discover capabilities/schema versions and authorized scope; inspect subjects, goals, structure and current revisions.
2. Search/outline originals; read relevant chunks and provenance. Build an inventory and missing-foundation hypotheses without inventing access or source content.
3. Create a persistent draft and stage candidate units/prerequisites/path plus cited explanations and a suitable mix of activities. Chunk a large course and keep a resumable work plan.
4. Read the effective draft, preview learner views and validate schemas/references/cycles/attribution/accessibility. Revise errors and label unsupported educational assertions. Request help/review for uncertain content rather than labeling it correct.
5. Submit a dependency-closed proposal. In default mode wait for UI review; in delegated mode apply only policy-allowed reversible groups, escalating the rest. Recheck bases/permissions/budget during each commit.
6. Inspect receipts and the resulting authorized structure. Continue improving content, propose a personalized practice plan if progress access exists, and leave a resumable job/draft/summary. Revoke or let the grant expire without blocking offline learning.

The same external host may perform these steps with a local model or a cloud-backed model; Memoquill does not prescribe a provider or contain a model runtime. The permission UI explains disclosure; service enforcement cannot control the host after data leaves it.

### Instructional design and assistance are separate from authority

Agents can author the [evidence-informed activity variants/phases](../research/learning-activities.md): examples/completion, explanation/teach-back, error repair and independent application, with declared outcomes, references, correction/rubric and help policy. Schema/source checks can detect missing fields/references and unmarked erroneous examples; they cannot certify educational truth. Learning-assistance studies do not imply a universal answer lockout or weaker authoring grants. Preserve rich delegated course authoring while distinguishing app-observed, learner-declared and unknown assistance in learner evidence; agents still cannot fabricate attempts/grades.

## Mandatory M6 qualification scenarios

- Build a multi-module English/Python course with citations, shared sources, prerequisites, several activity families and goals; repeat the same operations for a small math/cooking scenario without subject-specific exceptions.
- Perform many draft edits/validations/previews without manual clicks; disconnect/restart and resume from durable IDs/revisions. Chunk a plan beyond a single-request cap without unbounded allocation or losing attribution.
- Run both default exact-review mode and user-delegated automatic additions/organization. Verify budget/time/ownership limits, human-edit protection and sensitive-operation escalation; two parallel applies cannot double-spend a grant.
- Try nested out-of-scope references, guessed IDs, hidden duplicate/search leaks, forged approval/metadata, malicious source instructions, revoked grants, stale bases, payload limits and cancellation around commit. A resource/unit shared between granted A and denied B must not change B through an A-only grant; test explicit logical-object authority, safe fork/escalation and no hidden-metadata leak.
- Verify atomic groups, idempotent retry after lost response, no partial hidden application, compensating undo conflicts and no changed learner attempts/grades. Check original hashes and downstream history before/after. Revise an agent-created activity after the learner has studied it: protected semantic changes must request review or obey the explicit preselected relevance policy; they cannot silently preserve a misleading mastery/schedule interpretation.
- Test modern and named legacy client compatibility on the actual package; app-closed/stale-socket/error/recovery behavior; no unrestricted listener, filesystem or provider requirement.

Conformance, security and useful authoring must all pass. A large tool count or a successful connection is not evidence of a capable safe integration. M6 is an essential release milestone; “optional AI” describes user dependence/activation, not permission to omit the subsystem.
