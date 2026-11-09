# Post-M2 isolated courses: completed local integration

[KER-127](https://linear.app/kerigan/issue/KER-127), KER-128/129: implementation,
automated qualification and independent review complete. KER-130 foundation was
committed separately as c6a363e; catalogue foundation is 87648ff. This integrated
substep is committed locally in dev only. No push/merge/release or remote CI.

The existing workspace is adopted as one course, preserving nested Subjects,
Unit/lesson IDs, shared resource links, originals, versions, preferences and
drafts. New courses have independent native-derived database/asset/recovery roots.
Every data call requires explicit courseId and retains inner workspace authority.
Bound ports, mounted hidden/inert course trees and guarded focus/preferences keep
async results, file selection, edits and exact retries in their original course.
Catalogue initialization and shutdown fail conservatively rather than replacing
missing data or releasing ownership before writers drain.

Review found two Important issues, both reproduced RED and fixed GREEN: confirmed
creation rejection blocked navigation; absent/empty catalogue beside managed
course roots silently replaced ownership metadata. Scoped re-review accepted both
fixes and found no new Critical/Important issue. Minor deferred: dedicated v1
extension state/migration diagnostic tests; Vite still warns about the large
bundle (808 kB minified). No unrelated performance/dependency cleanup added.

Final normal gates: frontend 140 tests / 26 files; Rust library 173 passed,
3 existing ignored; linux_startup integration 4 passed, 0 doctests. Build,
fmt --check, all-target Clippy with warnings denied and diff checks passed.
Frontend coverage: 80.57% statements, 71.31% branches, 78.65% functions,
92.17% lines. Focused release GLib test passed before the final non-GTK fixes.
Logs: /tmp/memoquill-task2-fix-{frontend-final,native-final,build,clippy}.log.
Ignored review/checkpoint artifacts are under .superpowers/sdd/2026-11-08-course-workspaces/.
An intermediate test-discovery failure came from the parent's archived source
snapshot; it was moved outside the repository and ordinary final commands passed.

No GUI/screenshots or packaged course journey was run. Software completion is
distinct from M7 X11 keyboard/speech/reflow acceptance, release and educational
effectiveness. M3 and non-X11 work remain inactive; full visual design deferred.

Changed: native startup/commands/admission/catalogue lifecycle; bound ports,
CourseApp/provider/focus/preferences and behavioral tests; architecture/planning/
handoff documentation. Pre-existing AGENTS.md, .gitignore, workflow skill and two
November 1 reports are preserved and excluded from staging.

Decisions: retain dev/workspace as requested; adopt old data as one course (no
implicit split); cap v1 at 32 courses with recoverable denial; use one integration
worker plus independent review. Costs: explicit future split/scaling work and
review overhead. Interrupted initialization requires recovery rather than reset.

Linear was read back In Progress for KER-127/128/129 and Done for KER-130 during
implementation; completion changes use status-only writes/readback. Automatic
approval rejected appending private technical evidence to Linear; that disclosure
was not retried. All detailed evidence remains local in repository artifacts.

Used: engineering-workflow, tool-catalog, Memoquill workflow/context/routing/
reporting and learning-evidence guidance; Superpowers planning, TDD, execution,
debugging, verification/review and finishing. MCP: Serena, Codebase Memory,
Linear and Headroom; RTK/Git/Python/npm/Vitest/Cargo, apply_patch, collaboration.
Existing research reused; no fresh research, runtime plugin or new dependency.
Next: none within KER-127–130; no automatic M3 or GUI work.
