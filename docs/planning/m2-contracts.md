# M2 contracts and entry-gate disposition

> Current architecture clarification: see the composition section below. New feature/UI development is on hold pending architectural agreement.

Status: KER-109 contract baseline frozen; development-only desktop deferral explicitly dispositioned under the user's delegated judgment.
Updated: 2026-10-28 21:22 UTC.
Parent: [KER-99](https://linear.app/kerigan/issue/KER-99); first slice: [KER-109](https://linear.app/kerigan/issue/KER-109).

The user authorized starting M2 after preparation and asked the agent to inspect the desktop gate directly.
That supersedes preparation-only notes. It does not waive ADR-02 or KER-109 acceptance.
This document fixes reversible implementation choices; changing these requires updating the contract and its fixtures together.
No domain code, migration, manifest, installed desktop or runtime dependency has changed.

## Desktop disposition

Current local source is main8405103cae7bcafdeec15caebf2516bfcf0a035e. KER-98 is integrated; KER-97 remains In Review.
Source constraints are schema2, native-constructed RequestContext, trusted UI only, bounded single-writer queue, recovery guard and exact receipt/revision checks.

Direct environment probe on 2026-10-28:
- Environment reports XDG_CURRENT_DESKTOP=i3, XDG_SESSION_TYPE=x11, DISPLAY=:0, no WAYLAND_DISPLAY.
- Neither gnome-shell nor orca resolves in PATH; RPM reports both packages not installed; no matching process is visible.
- Sandboxed loginctl failed with Operation not permitted. The same read-only command through approved host execution succeeded: Remote=no, Desktop=i3, Type=x11, State=active.
- This is an actual local i3 session, not the intended physical Fedora GNOME/Orca environment. The direct probe does not establish a defect in Memoquill.
- Existing isolated Fedora X11/Wayland/IME/Orca presentation evidence is preserved. Repeating it would not satisfy human listening or physical GNOME acceptance.
- No global desktop switch, host package installation, settings change, audio capture or private application inspection was attempted.

**Disposition:** after these findings the user said the agent may decide how best to continue. Use that explicit authority for a bounded development-only deferral, recorded in ADR-02: retain Tauri2 and begin KER-110/KER-111 against the existing automated baseline. Leave KER-97 open and require physical GNOME/human Orca before acceptance. This is not a claim that the gate passed; an observed unresolved core failure triggers the existing fallback. No host security or desktop changes are authorized by the deferral.

Sources: [ADR-02](../architecture/decisions.md), [desktop baseline/manual checklist](../quality/fedora-desktop-baseline.md#gate-disposition-and-manual-review), [existing qualification harness](../../tests/qualification/README.md).
The baseline requires listening to results, Read last result, code/formula source, editing/IME, keyboard/focus and scaling in the intended session.
No known failing core case has been observed in this probe, so the Electron/GTK4 failure fallback is not triggered.

## 1. Identity, organization and ownership

| Record | Identity and invariants |
| --- | --- |
| Subject | Workspace-owned UUID; arbitrary user title, optional goals/prior knowledge, archived flag. No subject-kind enum or template requirement. |
| Goal | Stable UUID, belongs to a subject; title and explicit criteria text; no automatic mastery field. |
| Collection | UUID, one subject, optional same-subject parent, ordered child list. Containment is a tree; reject self/descendant moves. |
| Placement | Own UUID referencing an existing resource/unit; belongs to a collection. Reuse creates a placement, not a new resource identity. |
| Resource | Workspace-owned UUID/current revision; may appear under any number of subjects through placements. No exclusive current-subject ownership. |
| Revision | Immutable UUID, resource, monotonically increasing resource revision number, canonical document, schema version and provenance references. |
| Unit/path | Workspace-owned UUID; unit objective text/goal links/resource refs; paths are ordered unit references, with their own stable step IDs. |
| Relation | Stable UUID, typed prerequisite/related/reference edge. Prerequisite endpoints are units; no self-edge or cycle. Related/reference may cycle. |
| Annotation | UUID, body text, resource+immutable revision+block/quote target, explicit anchored/ambiguous/missing status. |
| Tag | Workspace-owned UUID/label and explicit membership. Derived search never becomes authoritative content. |
| Reading position | Personal workspace state: resource+revision+block+UTF16 offset and optional viewport hint; separate from content revisions. |

Subject, collection and path operations preserve resource/unit/revision/annotation identities.
Moving a collection subtree across subjects changes the subject of the complete subtree atomically; retains collection/placement IDs and content refs.
Reusing a subtree is not a move: an explicit copy creates new collection/placement IDs while retaining referenced resource/unit IDs.
No implicit deep content clone or learning-history transfer occurs.
Cross-workspace IDs are always rejected; future package import must map them explicitly.
Collection entries are an ordered sequence of placement IDs. Reorder requests provide a complete expected-current sibling sequence; reject duplicate/missing/foreign entries.
Path steps may repeat a unit intentionally but have distinct step IDs; prerequisite edges remain independent of display order.
Archive hides ordinary selection and retains references/revisions. No permanent purge or cascade deletion of learning history is introduced.

The shared resource impact set is every placement/unit/path that reaches it; edit UI must disclose sharing.
The current local UI is trusted for its workspace. Future external principals cannot assume a subject placement grants authority over every shared object.
M2 retains rejection of unverified external requests; it does not implement grants, a token system or MCP transport.

## 2. Canonical document version1

Storage is Memoquill JSON, not raw ProseMirror state or trusted HTML.
Envelope: schemaVersion, blocks. Every block has a unique stable UUID and an allowlisted type.
Represent paragraphs/headings, inline text/marks, lists, code with optional language label, authored math source, tables, managed media reference/caption/alt text and unsupported-content placeholders.
Inline marks are semantic emphasis/strong/code/link; link labels are text, hrefs follow an explicit scheme policy.
Code is passive text. Heading levels1–6; rectangular tables, no merged cells in version1.
Empty editable documents retain one paragraph block; empty strings are valid content.
Split creates a new block ID; merge retains the first block ID and marks affected old anchors for resolution.
Duplication allocates new IDs. Resource revision creation preserves unchanged block IDs.

Unsupported content is retained as opaque inert payload plus original-source/provenance reference and visible reason; it is never reparsed as active markup.
Unsupported future document versions are readable only as an explicitly limited original/placeholder; writes are refused without overwriting bytes.
Schema upgrade functions are deterministic, tested against prior fixtures and never execute supplied code or SQL.

Initial engineering budgets, requiring focused qualification rather than scientific claims:
- UTF-8 canonical document serialized size at most1MiB; at most10000 blocks/inline nodes combined; structural depth at most16.
- Text in any single block at most64KiB UTF-8; title/label at most256 Unicode scalar values; reject NUL.
- Formula source at most2048 Unicode scalar values and enforce an additional UTF-8 serialization budget; renderer maxExpand100/maxSize10/trust=false retained initially.
- Table at most100 rows by32 columns; count all cell descendants toward node/depth/size budgets.
- M2 attachment stream at most64MiB per explicit selection; no automatic remote download. Total storage exhaustion remains a recoverable failure.
- Search query at most256 scalar values, page size1–100 and bounded cursor/result work; result ordering has a stable ID tie breaker.
- Apply the total byte budget before full JSON parsing and enforce structural limits during validation. Frontend checks supplement Rust validation.

Documents preserve text/code/math whitespace and Unicode exactly; search normalization is derived only.
Serialized request objects use deterministic field order; maps are key-sorted for hashing, arrays retain order, optional/null rules fixed, no floating-point values in the version1 content contract.
Receipt hashing includes contract/version/workspace/operation kind and all payload/base revisions; excludes operation ID itself.

## 3. Anchors, revisions and editor lifecycle

Anchor tuple: resourceId, revisionId, blockId, startUtf16, endUtf16, exactQuote, optional context/hash.
UTF16 offsets refer to the preserved block string; Rust validates boundaries, never slices UTF-8 bytes using those numbers, and rejects offsets inside surrogate pairs.
Combining characters remain unchanged; selection expansion by grapheme is a UI operation and does not rewrite canonical coordinates.
An unchanged block/quote may be verified against the new revision. Multiple/no matches produce ambiguous/missing status; no silent nearest-position guess.
Original source anchors retain their exact source revision independently of authored-resource anchors.

Editor seam: canonical document in/out, validation/loss diagnostics, resource+base revision and dirty/saving/saved/conflict/unconfirmed state.
Editor adapter instances persist across callbacks, autosave status and unrelated React renders; switch resources explicitly and preserve dirty draft before disposal.
Undo/redo is local editor state, not rollback of acknowledged immutable revision history.
A lost IPC reply produces unconfirmed, then receipt lookup/exact retry; no blind overwrite.
Concurrent edits return a conflict with the current base identifier and retain the user's editable draft. No implicit last-writer-wins merge.
Save success requires durable service acknowledgement. A failed write or full queue never clears dirty state.
Reading resume on changed/deleted blocks offers an explicit fallback and does not imply reading/comprehension of the new revision.

## 4. Application and storage interface

Reuse ApplicationService; add a small family of typed organization/resource/annotation/navigation commands, not arbitrary SQL or caller-selected paths.
Native adapters construct RequestContext. Serialized caller payloads never contain trusted_ui or a principal.
Every mutation has contractVersion, workspaceId, operationId, expected base revision(s) and a typed payload.
Validate every nested reference and effective shared impact before applying; check invariants against the state inside the single-writer transaction.
Transaction contains graph/revision/current-pointer/tag updates plus exact request receipt.
Operations.kind schema2 CHECK supports only preferences/asset: introduce a checksummed migration that expands operation types and validates old receipt recovery.
Keep existing preferences/asset identities and behavior. Supported schema1/schema2 fixtures either migrate safely or leave the prior workspace usable.
No reverse SQL downgrade, no imported migration execution, no new connection bypass.

Surface outcomes: Applied with durable IDs/revisions; InvalidRequest with bounded field diagnostics; NotFound/archived-reference diagnostic; RevisionConflict; PermissionDenied; Busy; RecoveryRequired; OutcomeUnconfirmed.
Avoid revealing denied IDs or source payloads in error details/logs.
Attachment adapters accept selected native handles/streams; serve originals by checked managed ID with bounded reads.
Never add a generic path opener, remote URL fetcher, shell, HTML webview or permissive custom-protocol scope.
M2 basic search covers title/tag/canonical text with parameterized bounded matching; M3 owns ingestion/FTS pipeline qualification.
Define query tokenization/literal behavior before UI tests; no undocumented FTS query-language execution from a user string.

## 5. Four common-contract fixtures

| Fixture | Organization/material | Required checks |
| --- | --- | --- |
| Quantum physics | Blank subject, optional goal, complex-number unit prerequisite, two-state explanation, math/table/caption. | No template/category; canonical equation/source/table and anchor round trip; prerequisite cycle rejected. |
| English | Blank subject, Unicode text and shared explanation placed in two collections; original attached bytes. | Same entity types/forms; IDs/provenance survive reorder/reuse; no vocabulary-only schema. |
| Python | Blank subject, code/indentation/Unicode string, concept unit, related link. | Code never executes; exact whitespace round trip; related cycle accepted, prerequisite cycle denied. |
| Cooking | Blank subject, technique unit, ordered explanatory resource with caption and external source URL as data. | Same contracts; no implicit network; no quiz-derived safety/competence inference or privileged procedure domain. |

Fixtures are platform-validation examples, not scientific or safety advice.
M2 does not implement M4 activities, attempts, scheduling or assessments.

## 6. Acceptance-to-test seams

| Seam | Evidence required before child completion |
| --- | --- |
| Pure organization operations | T01 four-subject parity; randomized move/reorder/share; cycle/ownership rejection with unchanged state; stable IDs/refs. |
| Canonical codec/editor adapter | Golden round trips, bounds/unknown-node preservation, schema migrations, code/math/tables/captions and Unicode anchors. |
| ApplicationService | Receipt/retry/conflict, nested references, atomic graph updates, schema1/2 migration and rollback; no source/history loss. |
| Native attachment adapter | Selected handles, size/hash, path/type spoofing, passive original access, missing/corrupt bytes and publication failpoints. |
| User authoring/navigation | T10 save/error/draft retention, focus/selection/undo/IME, shared impact, tags/search, result pagination, restart reading resume. |
| Packaged Fedora desktop | Keyboard/Orca names/roles/status/editing, math/code/table alternatives, physical GNOME/human listening and Wayland runtime evidence. |
| Changed-code security | Targeted SAST, dependency/secrets/license review when relevant, all paste/drop/restore/render routes, native capabilities/CSP and denial budgets. |

## 7. Tool and workflow selection

Project workflow owns readiness/contracts and scope/reporting. Global codebase-design supplies interface vocabulary.
Global ub-mode is selected for authorized code after entry-gate disposition: behavioral red/green slices followed by Crapper/Dryer/Mutator checks on actual code.
implement/tdd/code-review were inspected as alternatives, not activated; no duplicate workflow or unsolicited commit step.
Use Codebase Memory Tier2/search/trace/snippets/coverage and Serena for structure; ColGrep for intent if needed, rg for literals/configuration.
Current graph coverage generation2026-10-28T21:17:19Z matches relied-on source; migration001 line12 partial was already read directly in preparation. No exhaustive negative/caller claim.
Context7 for concrete installed-version uncertainties; Semgrep/Trivy specialist skills loaded only before their actual use. No tool installation, paid research or security scans for this docs-only slice.
Serena is already activated and its manual read this conversation. No new subagents or local AI tasks were needed.
Read-only session properties required host execution; no host configuration changed.
One literal source search was blocked by a search-frequency hook; graph symbol/snippet/source checks supplied the evidence instead.

Next: implement KER-110/KER-111 under the M2 start authority and recorded development-only deferral. The user's subsequent instruction authorizes a PR without merge; verify the final candidate before committing/pushing the scoped PR branch. Merge and release remain unauthorized.

<!-- lesson-composition-clarification:start -->
## Contract freeze reopened for composition architecture — 2026-11-01

The user clarified manual learner-built lesson experiences, equivalent later MCP authoring, replaceable frontend code and scientific learning quality. The prior KER-109 freeze remains historical evidence; it does not define or approve the new composition/registry/interaction contract. Product feature/UI development and further GUI qualification are paused until the revised architecture is agreed.

KER-117 resolves lesson aggregate ownership against existing unit/resource/activity concepts and separates canonical content/provenance, activity/evaluation definitions, composition/presentation and learner history. It freezes component/action capabilities, versions/references/migrations/unknown-type behavior, proposal/state scopes and explicit resource budgets before implementation.

KER-118 defines replaceable views/controllers/application ports/native-test adapters plus a local-guest identity/session seam for future login. KER-119 owns validated shared-service composition persistence and exact revision/receipt behavior. KER-120 owns manual offline composition and the shared trusted renderer/preview; author previews are not real study outcomes. KER-116 now depends on these four tasks.

Acceptance includes four-subject manual composition, isolated lesson tabs/settings, safe custom methods, invalid/deep/cyclic/cross-scope inputs, unknown-version preservation, offline save/reopen, anchor/history continuity, exact retry/conflicts, semantic reading/keyboard/focus behavior and replacing a screen without Rust storage/domain changes.

M4 supplies real study/evaluation/history, M6 the live scoped agent adapter with manual/agent parity. Only a guest identity seam is prepared now; actual registration and cloud services remain future work. M8/KER-121 is a final default-off per-lesson gamification extension after core M7 qualification and does not block the core stage.

See [requirements](lesson-composition-requirements.md) and [research](../research/composable-learning-architecture.md). Existing source/provenance, offline, native security and physical GNOME/human Orca gates remain unchanged. No implementation completion, dependency choice or release authority is inferred.
<!-- lesson-composition-clarification:end -->

<!-- research-source-followup:start -->
## Follow-up contract constraints

Specify pinned content/activity/composition versions and safe regeneration transitions before real study, with history and question/evaluation semantics retained. Separate AI authoring provenance from actual learner-assistance evidence. Define metrics by meaning/unit/denominator/provenance rather than infer learning from reward totals, generation speed or UI preference.

Include lesson-scoped source-reference/bibliography/locator/coverage contracts now; KER-100 supplies real broad-format/book ingestion in M3 and KER-106 future source-to-learning proposals in M6. This is not a circular requirement to implement M3 parsing in M2. Source text cannot grant privileges or imply consent to remote upload. See [bounded screening](../research/composable-learning-screening.md). Application feature/UI/GUI development remains held for architecture agreement.
<!-- research-source-followup:end -->
