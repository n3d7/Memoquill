# M4 instructional-phase continuation — stopped at user request

2026-11-13. Full M4 remains approved: finish M4.1/KER-102 + KER-131,
then M4.2/KER-103. The user explicitly stopped work because quota is ending.
Do not resume until a new go-ahead. No GUI/screenshots, commit/push/merge,
publication, tracker writes or existing learner-content modifications performed.
HEAD: 7325467b0bb43c14477624055562d3ddd415c77b, branch dev.

## Implemented delta

- Constructed-response v2 pins explicit workedExample/completion material.
  Authoring and inert preview support both; study renders text without executing it.
  Ordered activities can provide example/self-explanation, partial completion and
  a separate independent problem. No automatic semantic grading was added.
- Practice persists shown solutions/scaffolds as solutionRevealed assistance;
  advancement sets assistance for the next instructional item and clears it for
  independent items/completion. Independent responses remain separate evidence.
  Assessment creation and imported-session admission reject any instructional
  item, including later items. Existing definitions are not migrated.
- Native catalog now describes both prior bounded-cloze v2 and constructed v2
  alongside the four v1 schemas. Four activity families remain unchanged.
- Fixed native study ACL integration: study_apply/read/sessions were registered
  in lib.rs but absent generated command manifest and main capability. Added only
  these three commands to build.rs and existing main.json; foreign windows and
  remote origins remain denied.
- New tests: instructional authoring/rendering/admission, restart across the
  example/completion/independent sequence, process termination before/after SQLite
  commit with same-operation replay, and headless native IPC across two courses
  with identical session IDs, malformed scopes and untrusted windows/origins.

## Evidence and corrections

- Initial quick passed/audit passed:
  .qualification/verify/20261113T204357-24e7da9b/summary.json.
  It preceded later tests and final ACL/catalog-test edits, so final quick is due.
- First focused run failed: two frontend paths incorrectly combined as one
  selector; native IPC exposed missing generated command registration.
  .qualification/verify/20261113T204515-2e96b383/summary.json.
- Next focused run: frontend passed; native failed because main capability also
  lacked the new command permissions. 260 passed / 1 failed / 5 helper skips,
  audit passed: .qualification/verify/20261113T204630-b20009c5/summary.json.
- A test-helper edit briefly removed a source range and caused Rust syntax failure:
  .qualification/verify/20261113T204734-4d41884a/summary.json. Restored every
  original test body verbatim from the initial backup; retained all new tests and
  replaced only setup helper. No unrelated source file changed in this correction.
- Corrected focused study run passed: 79 passed / 0 failed / 5 helper skips,
  audit passed: .qualification/verify/20261113T204842-0edcbfd5/summary.json.
- Broad Rust first failed the old catalog count assertion (6 actual versus 4
  expected): 273 passed / 1 failed / 5 helper skips, audit passed:
  .qualification/verify/20261113T204932-38fc8198/summary.json. Updated assertion
  to verify all six family/version identities and both new schema contracts.
  Rust-doc discovery had no tests; not a passed doctest suite.
- Final broad Rust passed: 274 passed / 0 failed / 5 helper skips, audit passed:
  .qualification/verify/20261113T205031-8323cc52/summary.json. This used affected
  --only rust; frontend, rust-doc and runner-self-tests omitted in this invocation.
  It had already finished when user stop was processed; no test was launched after
  stop. No verification process remains owned/running.
- Bounded independent review found one P2: future instructional items in imported
  assessment sessions escaped the current-item check. Fixed global invariant and
  added regression. Follow-up review of invariant, reset and main-only ACL found
  no material issue. Static review is separate from runtime qualification.

## Preservation and remaining acceptance

Initial verified 88-file backup: .qualification/m4-resume-preserve-20261113T183953Z/.
All baseline files outside the explicitly owned delta remained byte-identical.
New snapshot path is recorded in HANDOFF. Product source was not changed after
final passing Rust run; only this receipt/HANDOFF and ignored backup were written.

M4 remains incomplete, no issue is Done. Next authorized continuation: read current
HANDOFF and this receipt, verify changed Git facts, run final quick types/format
(one value per --frontend/--rust; --only repeatable), resolve only concrete failures,
and reconcile M4.1 criteria before beginning M4.2. Reuse frontend pass unless a
new frontend change/risk justifies repetition. Full M4 candidate still requires
full profile; native GUI, packaging/human, Sonar/licenses/CI/publication are not_run.
M4.2 dependency/adapter/review events, clock/day policies, workload/reminders and
frozen scoped assessments are not implemented. Existing FSRS evidence was read;
no dependency installed, external research performed or scheduler activated.

Tools/skills: Git, rg/bounded reads, Python/tar/SHA-256, Serena, Cargo fmt,
external verification runner, engineering-workflow, memoquill-learning-evidence,
one bounded independent reviewer. No observer or screenshot capture.
