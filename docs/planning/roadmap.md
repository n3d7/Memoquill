# Foundation V1 implementation roadmap

This document records architectural implementation order. Current phase: M2 architecture/research under KER-117; application feature/UI development and further GUI qualification are paused until the revised architecture is agreed. See HANDOFF and current Linear state; historical permissions do not cancel this hold.

Core Foundation spans M1–M7. The user-requested M8 is a final optional per-lesson gamification extension after core qualification, not a prerequisite for the core stage. No release/publication authority follows from the roadmap.

Historical M1 source/qualification preparation remains in its reports. KER-97 still needs physical GNOME/human Orca acceptance; automated presentation evidence is distinct. M2 has existing bounded implementation commits, but the newly clarified constructor is not implemented or accepted.

M1.2 [technical preparation](m1-2-workspace-preparation.md) is historical. KER-98 is Done in the inspected tracker; the manual KER-97 gate remains separate. Current M2 contracts and the architecture hold supersede old execution-status paragraphs.

| Order | Milestone | Dependencies | Verifiable capability / exit |
| --- | --- | --- | --- |
| M1 | Desktop foundation and durable application services | None | Packaged Fedora feasibility for Tauri/Orca/editor/math/IME; typed Rust/UI/service boundary; workspace/migration/asset/operation foundations, single-instance writer and safe failure. Decision gate before domain expansion. |
| M2 | Composable lessons, authoring and navigation | M1 | Existing organization/canonical authoring plus KER-117–120: composition/activity/view contracts, replaceable UI/controller/service ports and guest identity seam, shared validated persistence, manual offline composer/renderer. Architecture agreement precedes feature/UI work. |
| M3 | Lesson sources, ingestion, discovery and ownership | M2 | KER-100: lesson-scoped research/books/files, immutable originals and extracted revisions, qualified PDF/EPUB/common-format view/extract matrix, locators/coverage, safe reimport and search. KER-101: portable/native recovery with source bindings and explicit completeness. Retain conditional PDF and resource/security gates. |
| M4 | Activities, feedback, sessions, and recall review | M2 | Four activity families, assessments, exactly-once durable attempts/drafts, feedback/help evidence, FSRS recall adapter and separate practice reminders. Extend export/recovery with new history records. |
| M5 | Honest progress, personalization, and workflow completion | M3, M4 | Outcome-specific history/coverage/errors, accessible appearance/shortcuts/settings/saved views, consistent resumption and offline journeys. No universal mastery percentage. |
| M6 | Capable MCP authoring, scoped autonomy and reviewed changes | M3, M5 | Packaged stdio bridge/shared services, modern/legacy compatibility, discovery/schema/read tools, resumable draft authoring/staging/previews/jobs, explicit grants/revocation, exact transactional review or bounded autonomous apply/undo/audit. Three substantial issues separate transport/read, authoring, and authority/transaction risks. |
| M7 | Fedora release qualification and Foundation V1 | M1–M6 | Reproducible supported RPM, full backup/migration/crash/security/accessibility/scale/offline qualification, four-subject scenarios, reviewed English/Python example packs, notices and release documentation. No release/publication implied by passing local tests. |
| M8 | Optional per-lesson gamification | M7; shared M4–M6 contracts | KER-121: default-off lesson-local achievements, levels, scores/ratings, percentages/scales and bounded custom rules; transparent metrics, applicable evidence and learning/time/autonomy/accessibility checks. No global XP/account level; does not block core M7. |

M3 and M4 can proceed independently after M2 when ownership is separated; integration/format changes must stay compatible. M5 joins them. MCP contracts and authorization boundaries are designed now and considered in M1–M3 services, while the working adapter follows stable user workflows in M6. This prevents exposing unfinished raw storage as an integration shortcut.

The [additional learning-activity evidence](../research/learning-activities.md) refines M4/M5/M6 acceptance rather than adding milestones: shared instructional phases/presets, suitable feedback/error repair, clear observed/declared/unknown assistance and honest independent-check reporting. The four-family architecture and dependency order remain unchanged. A full native drawing engine, a separate teach-back service or universal pedagogical lockout is not added by these findings.

## Qualification decisions

- **M1 hard gate:** real Fedora packaged accessibility/runtime checks. If Tauri 2 cannot support core workflows after a bounded investigation, update ADR-02 before expansion; evaluate Electron/native alternatives with the same tasks.
- **M3 conditional gate:** passive PDF extraction/viewing with maintained licensed dependencies, resource limits, source anchors, and no active content. A failed gate reduces PDF to preserved/explicitly opened attachment and must update release messaging/journey C.
- **M4 gate:** preserve learning history across revision/schedule changes; FSRS applies only to recall. An algorithm test is not proof of educational efficacy.
- **M6 hard gate:** useful end-to-end course authoring plus service-level scope enforcement, default review and user-delegated reversible autonomy. Sensitive changes escalate; no unrestricted localhost API, permission self-grant or direct DB fallback.
- **M7 hard gates:** lossless included-data recovery, acknowledged-attempt durability, offline operation, core keyboard/Orca workflows, untrusted-input handling and correct licensing. Other usability/performance budgets require measured results and explicit disposition, never fabricated passes.

## Deferred work

Other platforms, cloud/sync/accounts/collaboration, autonomous in-app AI, executable programming sandboxes, unqualified OCR/complex-format extraction and full web crawling, advanced adaptive/psychometric assessment, a marketplace, and a visual programming/graph editor remain outside V1. A post-V1 idea needs a concrete user need and dependency review rather than being folded into an implementation issue unnoticed.

<!-- lesson-composition-clarification:start -->
## Architecture-first clarification — 2026-11-01

Feature/UI development and further GUI qualification are on hold until the revised architecture is agreed. KER-117–120 extend M2; KER-116 cannot close against the earlier fixed-form scope. M4 retains real study/evaluation/history and M6 exposes the same authoring operations to agents. M8/KER-121 is the user-requested final optional extension after core M7 qualification, with its own validation; no release/publication authority follows from this ordering.

Accounts remain future work. KER-118 prepares only a guest-first identity/session adapter seam; actual registration/provider/session design and cloud services are deferred. Core learning remains offline without accounts. See [requirements and task mapping](lesson-composition-requirements.md) and [research](../research/composable-learning-architecture.md).
<!-- lesson-composition-clarification:end -->

User clarification, 2026-11-01: EPUB/book reading and extraction are now explicit M3 qualification targets alongside PDF/common documents. Capability is reported honestly; broad formats are not a universal parsing guarantee. M2 defines source seams, M3 ingests, and M6 structures authorized selected sources into learning proposals. See [source/research admission](../research/composable-learning-screening.md).
