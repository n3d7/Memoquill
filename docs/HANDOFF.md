# Memoquill continuation checkpoint

Updated 2026-11-08 16:13 UTC. Branch dev; post-M2 source integration complete locally.

- Authorized scope: KER-127/128–130, completed substeps committed in dev only; no push/merge/history rewrite.
- Course catalogue foundation 87648ff; inert extension foundation c6a363e; integrated routing/UI follows in the local completion commit containing this checkpoint.
- Existing workspace adopted in place as one course; Subjects stay nested, preserving shared IDs/links/history. New courses use separate DB/original/recovery roots.
- Explicit courseId native routing and immutable bound ports cover content, sources, lessons, preferences, search and receipts. Unknown/unbound scope fails closed.
- Course sidebar creates/selects/persists selection; mounted hidden/inert trees retain edits, responses, pending exact retries and source selection identity. Inactive focus/preferences guarded.
- Missing initialized storage or missing/empty catalogue beside managed roots requires recovery. Never silently bootstrap replacement ownership or adopt unknown roots.
- Shutdown denies new work and releases catalogue ownership only after successful writer drain. Interrupted first initialization conservatively requires recovery.
- KER-130 data-only extension contract preserves bounded exact JSON, scope/capability/version diagnostics and passive fallback. No executable runtime, persistence/import route or simulators activated.
- Independent whole-change review found two Important issues; both reproduced and fixed, narrow re-review accepted. No open Critical/Important findings.
- Final checks: 140 frontend tests/26 files; 173 Rust library passed/3 existing ignored; 4 linux_startup integration tests; 0 doctests. Build/fmt/Clippy/diff checks pass; unchanged release GLib focused gate reused.
- Deferred minor: dedicated v1 extension state/migration diagnostic branch tests; Vite large-bundle warning remains. GUI/package journey and human speech/reflow NOT RUN here.
- Report: docs/task-reports/2026-11-08-161306Z-post-m2-course-integration.md; runtime: docs/architecture/course-workspaces.md; extensions: docs/architecture/course-extensions.md.
- Design/plan: docs/superpowers/specs/2026-11-08-course-workspaces-design.md; docs/superpowers/plans/2026-11-08-course-workspaces.md.
- Ignored ledger/review/report: .superpowers/sdd/2026-11-08-course-workspaces/. All owned workers/tests are stopped after completion.
- Linear status-only completion readback is recorded in the final response/ledger. Private technical description append was rejected by automatic approval and not retried.
- M2 PR #3 merged by owner; preserve all original commits. No new publication, remote CI or release.
- Main M1–M8 targets Linux/X11 only with portable contracts. Other-platform work stays separate/inactive after M8. Full visual design deferred.
- KER-97 manual evidence remains unaccepted; full X11 keyboard/speech/reflow stays M7. Headless/IPC checks are not human accessibility acceptance.
- M3/KER-100 sources preserve originals, ranges, provenance and honest coverage; M4 separates observed attempts from judgments; M5 learner goals/focus/format choices.
- M6 stays provider-independent: scoped MCP authoring/research proposals, ask-before-changing manual edits, opt-in course schedules. No actual transport, daemon or schedule started.
- Abruptly interrupted work waits for fresh permission, even after prior authorization; do not replay jobs from checkpoints.
- Existing learning-science/activity evidence reused; no new learning-efficiency claim.
- Preserve unrelated AGENTS.md, .gitignore, workflow-skill and November 1 tooling report changes; never stage/revert them.

Next: none within post-M2 KER-127–130. Wait for the user's next task; do not start M3, GUI or publication automatically.
