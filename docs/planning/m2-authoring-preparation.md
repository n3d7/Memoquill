# M2 authoring and navigation: preparation

Prepared 2026-10-28 for [KER-99](https://linear.app/kerigan/issue/KER-99/m2-deliver-custom-subjects-material-authoring-and-navigation).
**Phase: preparation and explicitly requested Linear decomposition only. Application implementation has not started.**
This is a proposed execution foundation; it does not authorize commits, pushes, publication, later milestones or a release.

## Verified starting point

- Local main HEAD: `8405103cae7bcafdeec15caebf2516bfcf0a035e`, merge PR #2. Initial working tree clean; origin/main tracking has no local divergence in current local refs. No remote fetch/CI requalification was performed.
- Linear KER-98 is Done; KER-99 is Backlog. KER-97 remains In Review. Its physical GNOME/human Orca acceptance is not closed by M1.2 source integration. Resolve or explicitly disposition the documented desktop gate before domain expansion.
- Source evidence: `src-tauri/src/workspace/service.rs:10-112` has a capacity-64 single-writer request interface, typed preferences/asset/receipt requests and context authorization before queueing. `operations.rs:59-95` checks exact receipts and expected revision before updating preferences and inserting its receipt. Reuse these patterns; do not infer that subject/resource operations already exist.
- `001_workspace.sql` contains workspace/preferences; `002_operations_assets.sql` constrains operation kind to preferences/asset. New operation kinds require a migration and typed outcome/recovery changes, not only new UI calls.
- Current ProseMirror fixture emits `textBetween` plain text, offers paragraphs/strong, intercepts paste as text and rebuilds its EditorView in an effect depending on initialText/onChange. It is a desktop feasibility fixture, not a structured canonical round-trip adapter. Preserve selection/undo/composition when replacing its data flow.
- Existing packages are pinned: ProseMirror model1.25.12/view1.42.8/state1.4.4 and KaTeX0.19.1. No dependency was installed or changed. Existing math preview already bounds input2048, maxExpand100/maxSize10, trust=false and MathML/source output; these values are current fixture behavior, not a complete M2 budget decision.
- Current main capability grants narrow workspace/preferences/receipt commands and dialog selection. Asset registration is a Rust-only seam. Attachments require deliberate native wiring; a picker is not a general filesystem capability.
- Current CSP disallows frames/objects, remote image sources and ordinary network connections. Preserve the policy; do not broaden it globally to make media/original previews convenient.

## Execution slices and Linear mapping

All children belong to KER-99, Memoquill Foundation V1 and its M2 milestone; all are Backlog, unassigned, without invented dates or estimates.

| Slice | Issue | Depends on |
| --- | --- | --- |
| M2.1 — Resolve desktop readiness and freeze M2 contracts | [KER-109](https://linear.app/kerigan/issue/KER-109/m21-resolve-desktop-readiness-and-freeze-m2-contracts) | KER-98 |
| M2.2 — Implement subject-independent organization and graph invariants | [KER-110](https://linear.app/kerigan/issue/KER-110/m22-implement-subject-independent-organization-and-graph-invariants) | KER-109 |
| M2.3 — Define canonical documents, immutable revisions and anchor contracts | [KER-111](https://linear.app/kerigan/issue/KER-111/m23-define-canonical-documents-immutable-revisions-and-anchor) | KER-109 |
| M2.4 — Persist organization, resources and revisions through typed services | [KER-112](https://linear.app/kerigan/issue/KER-112/m24-persist-organization-resources-and-revisions-through-typed) | KER-110, KER-111 |
| M2.5 — Build structured authoring with safe rendering and recoverable saves | [KER-113](https://linear.app/kerigan/issue/KER-113/m25-build-structured-authoring-with-safe-rendering-and-recoverable) | KER-111, KER-112 |
| M2.6 — Add managed attachments, provenance and passive original views | [KER-114](https://linear.app/kerigan/issue/KER-114/m26-add-managed-attachments-provenance-and-passive-original-views) | KER-111, KER-112 |
| M2.7 — Deliver accessible workspace navigation, tags and basic search | [KER-115](https://linear.app/kerigan/issue/KER-115/m27-deliver-accessible-workspace-navigation-tags-and-basic-search) | KER-110, KER-112, KER-113, KER-114 |
| M2.8 — Qualify integrated M2 integrity, security and Fedora accessibility | [KER-116](https://linear.app/kerigan/issue/KER-116/m28-qualify-integrated-m2-integrity-security-and-fedora-accessibility) | KER-113, KER-114, KER-115 |

Each child has concrete acceptance, tests, scope exclusions and the preparation-only authorization boundary.
M2.2/M2.3 can be designed independently after M2.1; M2.5/M2.6 share stable service/document contracts. This is a dependency graph, not authority to start parallel agents or coding.

## Model and interface decisions to freeze

Use small typed application operations with authoritative validation, explicit IDs, expected revision and operation identity. Hide storage/layout mechanics behind that interface; test it through the same entry points callers use. Do not create a second frontend implementation of graph or ownership invariants.

- Separate collection containment, ordered placement, logical resource, immutable revision, unit/path membership and typed relation. Multiple placements reuse content identity; related links may cycle; prerequisite and containment edges may not.
- Decide cross-subject move/share impact explicitly: a resource shown under A and B is not exclusively owned by the current screen. Validate all nested references and workspace membership. Future agent grants must eventually account for full effective impact; M2 adds no MCP runtime.
- Define deterministic reorder and move transactions (including concurrent stale requests), archive behavior, path duplicates, self-edges and target archival. Preserve history-bearing IDs and references. Do not implement permanent purge or speculative sync.
- Own the versioned canonical document schema in Memoquill. Relational identity/ownership is not opaque editor JSON. Specify stable unique block IDs, unknown-node preservation, schema upgrade/refusal and bounded parsing.
- Canonicalize payloads before request hashing. Define idempotent success vs stale expected revision, lost response vs failed transaction, and request-ID reuse with a different payload.
- Define source anchors in revision coordinates. Explicitly convert browser UTF-16 positions and Rust UTF-8 byte/scalar offsets; combining marks, emoji and normalization must not shift anchors silently. Never normalize original code/source bytes in place.
- Reading resume is resource+revision+block/position with a visible fallback on changed/missing blocks. A raw scroll offset is not a durable semantic anchor.
- Separate editor undo from durable operation undo. Specify save acknowledgement, dirty draft retention, resource switch/unmount and conflict retry; never mark saved merely because IPC was dispatched.
- M2 basic search must be useful offline and bounded. Define title/tag/text and query escaping semantics; M3 owns ingestion and full indexed discovery. Do not create a second indexing platform now.

## Security requirements and adversarial checks

These are design requirements and proposed tests, not confirmed vulnerabilities or a claim of application safety.

| Boundary | Required policy | Negative checks |
| --- | --- | --- |
| JSON/editor/service | Rust validates version, nodes, attributes, IDs, ownership and budgets; editor validation is supplementary. | Deep/huge JSON, duplicate block IDs, unsupported versions, bogus nested refs, malformed table spans and numeric bounds. |
| Paste/drop/render/restore | One allowlist policy across all routes. Render known passive nodes; code remains text. URLs are data, no implicit fetch. | Event attributes, active HTML/SVG/MathML, script/data/file/encoded schemes, CSS URL tricks, remote images/fonts, internal clipboard payloads and drag/drop bypass. |
| Math | Explicit trust=false, finite input/expansion/layout budgets and isolated mutable macro state; preserve readable source and recoverable errors. | Expansion/size abuse, cross-document macro state, unsafe URL commands and failure announcements. |
| Native IPC/capabilities | Main app calls typed services; untrusted document view never gets general invoke/file/network access. | Forged caller context, guessed resource IDs, path injection, stale revision, queue saturation and malicious original view. |
| Attachments | Selected bounded streams, managed immutable IDs/digests, separate provenance records, passive previews and ownership checks. | Symlink/path escape, MIME/extension spoof, huge streams, corrupt/missing bytes, duplicate bytes with different attribution, interrupted publication. |
| Durable mutations | Domain checks and revision/receipt/reference changes in one transaction; schema migration preserves prior workspace. | Cycle/race rollback, lost reply/exact retry, hash conflicts, disk-full/busy, unsupported schema and incomplete recovery. |
| UI state/diagnostics | Keep edits on errors, explicit save/conflict status, no content/secrets in logs. | Close/switch during save, stale autosave completion, accidental diagnostic source disclosure, hidden metadata in counts/previews. |

Opening an unsupported original externally is a separate deliberate action and trust boundary, not a license to embed active content.
No generic file opener, shell runner, server, remote media fetch, importer/extractor, dynamic activity plugin or broad capability is added by this preparation.

## Evidence reviewed and bounded extension

Reuse [learning science](../research/learning-science.md), [learning activities](../research/learning-activities.md), [educational UX](../research/educational-ux.md), [technical options](../research/technical-options.md), [learning model](../product/learning-model.md), [journeys](../product/user-journeys.md), [data model](../architecture/data-model.md) and [technical design](../architecture/technical-design.md).
They support subject-independent contracts, source/history separation, readable status and user-controlled navigation; their engineering proposals are not measured Memoquill learning outcomes.

New [bounded evidence](../research/m2-authoring-evidence.md) verifies official editor validation/paste and math options. It records one self-hosted search, two useful pages and one GNOME404; moving docs still need installed-source verification at implementation time.
One additional public search and one author-hosted paper read checked the organization/learning distinction: Blunt & Karpicke (2014), [Learning With Retrieval-Based Concept Mapping](https://learninglab.psych.purdue.edu/downloads/2014/2014_Blunt_Karpicke_JEDP.pdf), DOI10.1037/a0035934. The abstract reports two experiments comparing retrieval formats and studying; on a one-week verbatim/inference assessment, the two retrieval formats did not differ and both outperformed further study. This bounded finding does not prove equivalence across subjects, accessibility or long-term practical competence. Product inference: collection/prerequisite structure is organization, not mastery evidence; M2 must not infer learning from authoring/navigation. It does not justify adding a concept-map canvas or M4 practice now.

## Skills, tools and plugins

Workflow owner for this phase: project `memoquill-task-workflow`, with routing/reporting for their distinct responsibilities.
Actually applied global skills: `skill-catalog` for focused selection, `codebase-design` for small authoritative interfaces/test seams, `research` for a bounded primary-source research agent. Imported implementation/TDD/review workflows were not started.
During authorized execution select one coding workflow, then load only its documented dependencies; the previous M1.2 workflow does not silently select M2's owner.

| Need during execution | Selected route | Conditions |
| --- | --- | --- |
| Structure/impact | Codebase Memory search/trace/snippets + coverage | Include tests and interface/implementation targets where relevant; no negative claims from empty search. |
| Intent/literals/edits | ColGrep / rg / Serena | Scope semantic queries; preserve bounded GPU wrapper; Serena already activated/manual read. |
| Exact library questions | Context7, then installed source/focused tests | Only public API questions leave the repository; not called in this preparation. |
| Security validation | semgrep-local + trivy-local; exact-diff security review if selected | Load skills before actual use; scan actual changes, new dependencies and lockfiles; no broad audit solely because tools exist. |
| Review | code-review when a concrete diff/spec exists | Requires its two review axes; not run against a nonexistent M2 patch. |
| Desktop/editor | Existing Fedora qualification harness + human keyboard/Orca | Actual packaged WebKitGTK/Wayland/IME/math/tables; jsdom/axe is complementary. |
| Public evidence | self-hosted Firecrawl; bounded web lookup | No paid deep research, Consensus job, uploads or infrastructure changes. |
| Tracker | Linear plugin | Reads and explicitly authorized child creation/parent decomposition only. |

Actual integrations: Serena activation/manual; Codebase Memory status/search/trace/snippets/coverage; Linear reads/writes; self-hosted Firecrawl in research agent; web search/paper read. RTK shell, Python skill catalog, apply_patch and a bounded research subagent were used.
Local AI, ColGrep, Context7, Semgrep, Trivy, Dependency-Track, SOFA, native UI automation and runtime application tests were not used; availability is not asserted.
Linear get_agent_context is exposed but returned tool-not-found; no shared notes retrieved. Headroom retrieval returned unresolved markers; targeted source reads were used where compressed output hid material facts.

## Verification and next action

Codebase graph: Tier2, generation2026-10-28T20:45:00Z, ready. Exact coverage checks for relied-on source/config paths had matching metadata/no recorded issue, except migration001 line12 parse_partial; entire migration including that range read directly. Coverage is best effort, not completeness proof. Source fallback verified service/editor/math/capability/migration facts. The graph provided editor inbound App and no outgoing edges; this is not a claim that external library calls are absent. Initial preference query was truncated (10/15); used only named positive candidates, no exhaustive claim.

Application tests, native runs, new scans and dependency installs: not run, because only documentation/tracker preparation was authorized. Verify documentation paths, unchanged application source and actual Linear child relationships before closing preparation.
Next action, after explicit implementation authorization: KER-109 desktop-readiness disposition and contract freeze. No M2 implementation begins automatically.

