# Verification strategy

This is a future engineering/evaluation plan. No application tests, accessibility qualification, usability study, retention experiment, or performance benchmark has run during the planning assignment. Documentation validation and source verification must be reported separately.

## A — Scientific justification

The [learning evidence register](../research/learning-science.md) connects finding → evidence quality → applicability → product decision → expected benefit → validation. Major choices include eligible spaced recall/retrieval with feedback, learner-paced explanation, novice examples/scaffolding, conditional interleaving, and explicit self-assessment limits.

Evidence for a mechanism does not prove this implementation, a particular interval, or learning gains for every population/subject. Do not turn FSRS benchmarks, engagement, faster navigation, or higher review accuracy into a claimed learning percentage. Contradictory/context-specific math, classroom, transfer and feedback findings remain in the register. Engineering hypotheses (selection mix, budgets, dashboards) require product evaluation and are labeled accordingly.

Future learning-outcome work, if separately authorized, should use suitable delayed assessments, new problems/contexts, practical observations where feasible, comparable time/material/difficulty, recorded prior knowledge, and assistance tracking. Distinguish recall, conceptual transfer, production, and real-world performance. Small exploratory user observations do not establish causal efficacy or population effects. No original scientific experiment is required by this assignment or V1 release.

The [additional activity evidence](../research/learning-activities.md) supports multiple response formats and configurable instructional phases. Format completion, confidence, AI-assisted performance and later independent performance remain different outcomes. The [Consensus audit](../research/consensus-review-audit.md) records verification limits and corrections; its generated search funnel is not our systematic-review evidence.

## B — Technical verification

Use a modest stack: Rust unit/integration tests, property tests where invariants benefit, TypeScript component tests for meaningful interaction logic, a small set of end-to-end packaged workflows, and focused fuzz/security cases at untrusted boundaries. Candidate tooling: `cargo test`, proptest, Vitest/Testing Library, axe-core, and a packaged Linux driver supported by the selected Tauri release. Pin/check compatibility during M1; browser-only Playwright runs do not prove native WebKitGTK/AT-SPI behavior.

| Area / gate | Required evidence and failure cases |
| --- | --- |
| T01 Domain / organization | Four subjects; arbitrary depth/multiple placements; stable identities under reorder/move; prerequisite/containment cycle rejection; permissions/ownership across references. Property tests for graph/order invariants where useful. |
| T02 Database / migrations | Every supported schema fixture migrates or fails safely; checksums/version rejection, foreign keys on each connection, constraints/indexes, concurrent reads/single writer, disk-full/busy/crash fault cases. No acknowledged attempt loss. |
| T03 Assets / backup / restore | Crash between staging/rename/DB commit, missing/corrupt asset, interrupted copy, WAL-active snapshot, backup lease vs GC, old/new schema packages. Full journey G in a clean workspace, hash/count/semantic comparison and failed-restore rollback. |
| T04 Import / reimport / export | Core formats with Unicode/multilingual/code/math/tables; invalid schemas/encoding, stable source keys, exact duplicates, partial failures/count reconciliation, cancellation/retry, human-edit conflicts, unknown types, source anchors, round-trip canonical records and originals. |
| T05 Activities / attempts / schedule | Each family/assessment mode, hint/reveal state, exact-once retries, draft resume, correct/incorrect/unknown vs self-rating, review event atomicity, algorithm/version fixtures, timezone/DST/clock rollback/long delays/overdue queues, invalid ratings. Non-recall never receives inferred memory competence. |
| T06 Search / scale | Rebuild from canonical data, update/delete/archive consistency, filters/permissions, stable pagination, representative languages and code, query escaping/limits, documented large fixture. Measure startup/search/interaction budgets and memory rather than inventing results. |
| T07 MCP / authorization | Modern/legacy protocol fixtures and SDK conformance; real-client course authoring/draft resumption. Every tool/resource scoped; rich schema discovery/preview/edit/batches/jobs, default review and delegated reversible apply, protected human edits/sensitive escalation, budget double-spend and revoke races, guessed IDs/nested refs/progress denial, forged approval, stale bases/hash mismatch, transaction rollback/idempotency/undo conflicts, limits/cancellation/process exit/app closed. No HTTP listener or implicit fetch. |
| T08 Untrusted inputs / renderer | Traversal/absolute paths/symlinks, archive bombs, oversized/deep/truncated payloads, malicious HTML/URLs/math/PDF instructions, fake MIME, renderer sanitization/CSP/invoke isolation, parser time/memory cancellation. Dependency/advisory/license review for selected binaries/libraries. Never execute imported code. |
| T09 Offline / Linux packaging | Fresh supported Fedora installation, RPM upgrade/uninstall-data policy, permissions/XDG paths, no network/core credentials, Wayland and supported X11 smoke, IME/clipboard/file dialogs, crash/restart, actual packaged webview/render/editor version. AppImage/Flatpak only if separately qualified. |
| T10 Accessibility / customization | Keyboard-only all core journeys, logical focus/return/error focus, dialogs/shortcuts/reset, text scaling/reflow, contrast/non-color status, reduced motion, math/code/tables; Orca reads name/role/value/status in packaged workflows. Automated accessibility checks supplement manual assistive-tech evidence. |

Release-critical integrity, authorization, offline, and accessibility failures block release. T03/T04 must exercise the native/portable inclusion matrix, complete/incomplete linked-source backups, and both G1/G2 paths. Pair PDF fixtures with extraction enabled and unavailable: attachment-only status, scoped original bytes, and no invented extracted anchors must work in the fallback. T07 must test a resource/unit shared by granted A and denied B: authorize its full effective impact or use explicit shared-object authority/fork/review without leaking hidden metadata or transferring mastery. Parser fuzzing/property suites should focus on real boundaries, not mirror implementation details. Do not build a large redundant test infrastructure or repeatedly run unchanged full suites. M1 establishes fast checks; each milestone adds tests for its risks and M7 runs the complete release qualification.

Additional T05/T07 fixtures must preserve phase/attempt/reveal state across resume; separate observed help, declared help and unknown external assistance; cover choice questions requiring inference alongside uncued production; and label deliberately flawed examples with a correct resolution. Verify example-first and bounded attempt-first paths without a forced universal hint delay. Structural source/rubric validation must not mark AI educational claims factually verified or reduce already-authorized MCP authoring capabilities.

### Large-collection fixture and performance

Use deterministic generated non-sensitive fixtures: 100 subjects, 10k resources, 100k activities, 1M attempts and 10 GB representative assets. Record Fedora version, hardware/RAM/storage, webview/runtime/package, cold/warm cache, indexes and sample count. Initial budgets from P-spec: useful warm start ≤3 s, common search p95 ≤500 ms, no >100 ms synchronous UI work during imports. Profile failures before selecting a second database/framework. These are proposed targets; no result exists yet.

Joint T06/T10 qualification on that fixture must let keyboard/Orca users reach results beyond the first viewport/page, with meaningful position/count announcements. Test beginning, middle and end; loading, sorting, archiving/deleting, and returning must retain sensible focus/selection without unreachable virtualized items. A fast accessible first page alone does not pass.

### Evidence/traceability

Each milestone issue records commands/environment/revision, observed outcomes, failure artifacts and checks not run. T01–T10 map to P01–P14 and [journeys A–G](../product/user-journeys.md); all four subject examples must pass the same domain/service contracts. A passing unit suite alone cannot satisfy packaging/accessibility or restore gates.

## C — Usability and workflow evaluation

Use formative task sessions before release, first a small 5–8 person round spanning unfamiliar users and experienced collection users, with keyboard/assistive-tech participants where feasible; revise serious failures and repeat affected tasks. Such a sample finds problems, not a population success percentage. A later 10-person checkpoint can apply the provisional task gates in [UX research](../research/educational-ux.md), reporting counts, observed barriers and sample limitations rather than a confidence-free universal claim.

| Task | Observe / provisional acceptance intent |
| --- | --- |
| Start/resume | Returning user reaches Resume/Due review in one action from start; measure time from useful view to first activity separately from launch. No setup requirement. |
| Blank subject A | Create custom organization and goal without a template; record confusion, navigation steps, assistance and recovery. Target ≥8/10 unassisted at later checkpoint. |
| Dataset B | Correct a mapping error, inspect originals/attribution, apply partial import/retry; no lost rows or silent duplicates. Measure completion/failure recovery, not only import speed. |
| Proposal C | Explain scope/host disclosure, inspect sources/diff, reject/edit/apply and resolve a conflict. Test default review and bounded autonomous reversible authoring separately, including escalation and draft/job resumption. Any permission self-grant or scope misunderstanding causing disclosure is critical. |
| Offline D / practical E | Complete a meaningful session with network disabled; interpret procedure self-report versus quiz evidence correctly. No misleading mastery/certification label. |
| Personalization F | Change/reset appearance/shortcuts and restore accessible navigation; observe discoverability and mistaken scope changes. |
| Recovery G | Export/backup, reinstall/restore, recognize corruption and content-only limitations. All canonical included data verifies; serious data-loss risk blocks release. |

Measure task success, assistance, time/navigation effort, mistakes/recoverability, perceived complexity, interruption/resumption, and keyboard barriers. Separate interface-operation time from intentional thinking/practice time. Satisfaction or sustained use can inform design but is not a learning outcome.

## Current planning-assignment checks

Before marking research/planning complete: verify required documents/local links, source traceability and access limitations, current MCP/version/licensing claims, scope/dependency consistency, Linear write results/IDs, existing-change preservation, final diff/new files, no app implementation and no commits/push. Report integration failures precisely even if a fallback succeeds. Future acceptance gates remain pending and are not presented as passed.
