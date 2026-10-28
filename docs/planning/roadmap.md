# Foundation V1 implementation roadmap

This document is the source of architectural implementation order. The backlog mapping carries exact issue definitions and actual Linear identifiers. No milestone has been implemented or started by this research/planning assignment. Implementation requires a subsequent explicit instruction.

Seven milestones deliver the smallest coherent Memoquill experience, rather than a runnable shell. No calendar estimates or artificial urgency are asserted. High priority reflects dependency/data/security impact; a blocked high-priority issue does not authorize bypassing its predecessors.

Current execution: M1.1 has a local RPM and passing bounded Fedora X11/Wayland/IME/Orca automation; see the [candidate baseline and remaining manual acceptance](../quality/fedora-desktop-baseline.md). It is ready for review, not a release or all of M1. M1.2 remains unstarted; no dependency or subsequent implementation authority changes.

| Order | Milestone | Dependencies | Verifiable capability / exit |
| --- | --- | --- | --- |
| M1 | Desktop foundation and durable application services | None | Packaged Fedora feasibility for Tauri/Orca/editor/math/IME; typed Rust/UI/service boundary; workspace/migration/asset/operation foundations, single-instance writer and safe failure. Decision gate before domain expansion. |
| M2 | Subject organization, material authoring, and navigation | M1 | Blank subjects, arbitrary collections/shared resources/units/paths/relations; versioned document editor, attachments/source view, keyboard/search navigation. Four-subject model with stable IDs. |
| M3 | Ingestion, provenance, search, and ownership | M2 | Core import preview/retry/reimport; preserved originals/anchors/attribution; FTS; portable package and native backup/restore. Resolve bounded PDF gate or ship explicit attachment-only behavior. |
| M4 | Activities, feedback, sessions, and recall review | M2 | Four activity families, assessments, exactly-once durable attempts/drafts, feedback/help evidence, FSRS recall adapter and separate practice reminders. Extend export/recovery with new history records. |
| M5 | Honest progress, personalization, and workflow completion | M3, M4 | Outcome-specific history/coverage/errors, accessible appearance/shortcuts/settings/saved views, consistent resumption and offline journeys. No universal mastery percentage. |
| M6 | Capable MCP authoring, scoped autonomy and reviewed changes | M3, M5 | Packaged stdio bridge/shared services, modern/legacy compatibility, discovery/schema/read tools, resumable draft authoring/staging/previews/jobs, explicit grants/revocation, exact transactional review or bounded autonomous apply/undo/audit. Three substantial issues separate transport/read, authoring, and authority/transaction risks. |
| M7 | Fedora release qualification and Foundation V1 | M1–M6 | Reproducible supported RPM, full backup/migration/crash/security/accessibility/scale/offline qualification, four-subject scenarios, reviewed English/Python example packs, notices and release documentation. No release/publication implied by passing local tests. |

M3 and M4 can proceed independently after M2 when ownership is separated; integration/format changes must stay compatible. M5 joins them. MCP contracts and authorization boundaries are designed now and considered in M1–M3 services, while the working adapter follows stable user workflows in M6. This prevents exposing unfinished raw storage as an integration shortcut.

The [additional learning-activity evidence](../research/learning-activities.md) refines M4/M5/M6 acceptance rather than adding milestones: shared instructional phases/presets, suitable feedback/error repair, clear observed/declared/unknown assistance and honest independent-check reporting. The four-family architecture and dependency order remain unchanged. A full native drawing engine, a separate teach-back service or universal pedagogical lockout is not added by these findings.

## Qualification decisions

- **M1 hard gate:** real Fedora packaged accessibility/runtime checks. If Tauri 2 cannot support core workflows after a bounded investigation, update ADR-02 before expansion; evaluate Electron/native alternatives with the same tasks.
- **M3 conditional gate:** passive PDF extraction/viewing with maintained licensed dependencies, resource limits, source anchors, and no active content. A failed gate reduces PDF to preserved/explicitly opened attachment and must update release messaging/journey C.
- **M4 gate:** preserve learning history across revision/schedule changes; FSRS applies only to recall. An algorithm test is not proof of educational efficacy.
- **M6 hard gate:** useful end-to-end course authoring plus service-level scope enforcement, default review and user-delegated reversible autonomy. Sensitive changes escalate; no unrestricted localhost API, permission self-grant or direct DB fallback.
- **M7 hard gates:** lossless included-data recovery, acknowledged-attempt durability, offline operation, core keyboard/Orca workflows, untrusted-input handling and correct licensing. Other usability/performance budgets require measured results and explicit disposition, never fabricated passes.

## Deferred work

Other platforms, cloud/sync/accounts/collaboration, autonomous in-app AI, executable programming sandboxes, OCR/EPUB/full web crawling, advanced adaptive/psychometric assessment, a marketplace, and a visual programming/graph editor remain outside V1. A post-V1 idea needs a concrete user need and dependency review rather than being folded into an implementation issue unnoticed.
