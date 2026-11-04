# M2 execution ledger

Authorized: user starts KER-99, delegates entry-gate judgment, requests one commit per completed child and a PR without merge.
GitHub CI is not awaited between child commits. Required local behavior checks still precede completion claims.
Branch: feat/ker-99-authoring; starting main8405103cae7bcafdeec15caebf2516bfcf0a035e.

| UTC / slice | Action and reason | Evidence / state |
| --- | --- | --- |
| 2026-10-28 21:26 / KER-109 | Refreshed project/global skills, source and tracker; inspected desktop directly, froze contracts and recorded explicit development-only deferral. | [contracts](m2-contracts.md), ADR-02; KER-109 Done. Physical GNOME/human Orca still mandatory; KER-97 In Review. |
| Next / KER-110, KER-111 | Implement pure organization and canonical document contracts with behavior tests, then integrate typed persistence. | In Progress; no downstream completion claim. |
| 2026-10-29 / KER-110 | Completed subject-independent organization, shared placement identities, subtree moves, order/archive/reference invariants and prerequisite DAG with atomic errors. | 16 domain behavior tests including250 deterministic moves; integrated instrumented98pass/3ignored. Pure-module Crapper max9.1 with fresh LCOV, Dryer no candidates>=0.8. Mutation raw214killed/0survived; audit excludes84 invalid tuple-field mutations and one uncovered tuple-field candidate, leaving130 mapped semantic operator kills across KER-110/111. No defect-freedom claim. |
| 2026-10-29 / KER-111 | Completed version1 canonical document codec, immutable revision/anchor types, budget/schema/URL validation, inert unsupported-source preservation and ProseMirror round-trip adapter. | Rust document11pass; adapter golden test and component fixtures pass. Unicode/UTF16 boundaries, overlapping quote ambiguity, malformed IDs/versions/limits are covered. Same audited pure-module metrics/mutations as KER-110; no importer/HTML execution or invented prior persisted schema migration. |

Workflow: project scope/reporting; ub-mode coding owner, codebase-design for interfaces. Its context rule sends bulk work out of the main thread; bounded workers may own independent modules with source/graph evidence and no overlapping edits.
Crapper/Dryer/Mutator checks apply to changed code after local behavior checks. Load relevant tool contracts before use.
Pre-existing this execution: preparation/HANDOFF documents from prior authorized task. Retained and included in KER-109 documentation commit.
2026-10-29 branch steering supersedes the earlier PR-without-merge rule: user makes dev the development branch, requests current work integrated there and all further substep commits there. Local dev tracks origin/dev; integration of existing M2 commits was fast-forward with44 working-file hashes verified unchanged. Merge dev into main only after the entire M2 step and required gates. Release and host desktop configuration changes remain unauthorized. Tracker writes stay scoped to Memoquill.


| 2026-11-01 / KER-112 | Completed typed relational storage/schema3, immutable documents and anchored notes/reading position; hardened persisted reads and native authority. | [Storage report](../task-reports/2026-11-01-095407Z-m2-typed-storage.md); 102 Rust tests pass/3ignored; scoped CRAP<=9.3 and six selected valid mutations killed. Final M2 acceptance remains pending. |

| 2026-11-01 / KER-113 component | Completed stable structured editor and nested authoring controls; fixed label-change focus/history loss. | [Editor checkpoint](../task-reports/2026-11-01-100313Z-structured-editor.md);14focused tests, TS, scoped CRAP<=9 and20valid selected mutations killed. Integrated save/native acceptance remains in progress. |

| 2026-11-01 / KER-114 backend/native | Immutable managed originals, provenance, schema4 and bounded passive preview; native chooser copies regular files without caller-path authority. | [Managed originals checkpoint](../task-reports/2026-11-01-101213Z-managed-originals.md);116Rust tests/3ignored; scopedCRAP<=9,8selectedvalid mutationskilled. UI/native acceptance remains pending. |

| 2026-11-01 / KER-114 component | Added passive originals/provenance component with exact pending retry, cancellation focus and draft protection. | [Source UI checkpoint](../task-reports/2026-11-01-101541Z-source-authoring-ui.md);10tests/TS, scopedCRAP8.323,8selectedvalidmutationskilled. Production/native journey remains pending. |

| 2026-11-01 14:37 UTC / workflow routing | User selected Superpowers as primary; ub-mode only selected quality checks. Added project engineering router/examples and explicit skill-catalog link. | Local skill validation/catalog/links pass; new .agents files ignored by existing user rule, Git inclusion preference pending. Native collection failure traced to text-vs-accessible-button test predicate; application write visible in screenshot. |

| 2026-11-01 16:02 UTC / global tooling and M2 continuation | Personal engineering router promoted globally; tool-catalog replaces skill-catalog with compatibility symlink, nine purpose files and skills/MCP/CLI/built-in metadata. Local-only preference resolved. | [Catalog report](../task-reports/2026-11-01-155430Z-tool-catalog.md); 20 tests/validator/check pass. M2 59 frontend/120 Rust tests (3 ignored), Clippy and current RPM build verified. Integration review underway; KER-115/116 In Progress. Request screenshots from user; do not capture. Native/physical gates and main integration remain open. |

| 2026-11-01 19:43 UTC / architecture hold | User clarified manual/agent lesson composition, frontend replacement, scientific learning quality, optional lesson-local gamification and future identity seams; paused application feature/UI/GUI work. | KER-117–120 added; KER-116 dependencies expanded; final M8/KER-121 added; M2/M4/M6/M7 tracker/docs synchronized. [Research](../research/composable-learning-architecture.md), [requirements](lesson-composition-requirements.md), [report](../task-reports/2026-11-01-194306Z-architecture-research-roadmap.md). No product implementation or new-architecture acceptance. |

| 2026-11-01 20:11 UTC / selected research and source ingestion | Native arXiv/Consensus screening admitted only specific control/measurement boundaries; user added lesson research/books/files. | [Admission ledger](../research/composable-learning-screening.md), [follow-up report](../task-reports/2026-11-01-201122Z-research-source-ingestion-followup.md). KER-100/M3 and KER-106/M6 own actual source intake/synthesis; M2 source seams avoid a dependency cycle. Product source/tests/GUI remain held. |
