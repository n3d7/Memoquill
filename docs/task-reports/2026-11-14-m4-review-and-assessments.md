# M4 implementation and local qualification

User resumed full M4 on 2026-11-14. No GUI/screenshots, commit/push/merge,
publication, tracker writes or existing learner-content changes. HEAD stays
7325467b0bb43c14477624055562d3ddd415c77b. All 92 prior checkpoint files were
unchanged at resumption. Preserve earlier dirty work and the previous checkpoint.

M4.1 final quick passed/audit passed at .qualification/verify/20261114T001015-1ee45633/summary.json.
Earlier broad Rust 274/0/5 helper skips and frontend pass remain in the previous
instructional receipt. Milestone/native/human acceptance is separate.

## M4.2 implemented so far

Pinned fsrs =7.0.0, jiff =0.2.38, jiff-tzdb =0.1.9. Exact public FSRS archive
SHA-256 5cfbc88edd1cc96cec10317192b57cc5a5c50bd48f3144d3920b9bf5da87fa69
matches Cargo.lock. Full BSD-3-Clause license and all 34 FSRS-7 parameters read
from published source/archive. No optimizer/network/runtime AI activated. Explicit
bundled TZif loading avoids host timezone lookup. Linux dependency closure and
actual notice texts regenerated; license evidence passed. Evidence and adapted
one-off notice collector: .qualification/m4-fsrs-dependency-evidence/. No prior
collector or unrelated tool modified.

Review module, schema9 migration, service/native main-only ACL and UI support:
explicit rated unassisted independent/delayed recall; atomic attempt/event/state/
receipt/cursor; persisted parameters/hash/algorithm/generation/timezone rules;
clock rollback rejection; civil study days/DST; daily budgets; separate practical
manual/fixed reminders and operation receipts. Optional bindingIds freeze selected
recall sessions. Portable studyHistory2 carries fixed-whitelist review rows;
legacy history1 and native schema7/8 admission remain supported. New scheduled
attempts cannot be admitted as schema8. No existing learner data opened/changed.

## Verification and corrections

- Cargo check --lib passed on pinned Rust 1.98.1. One preexisting test-only
  capture_portable dead-code warning remains to address if final Clippy requires it.
- Initial quick blocked by stale license inventory; collector had accidentally
  included unrelated target-platform packages with missing notices. Changed
  hypothesis to reachable Linux normal/build/test graph (333 packages), refreshed
  real notices/inventory; direct license evidence passed. Failed receipts:
  20261114T002955-3118774e and 20261114T003026-6569b04c (second refused unchanged
  failing inputs, not a test pass). Quick then passed/audit passed:
  .qualification/verify/20261114T003214-62663d77/summary.json.
- Focused review_ Rust passed 11 tests; study frontend initially 10 pass/1 fail
  because a new test used the wrong self-rating label. Overall failed receipt:
  .qualification/verify/20261114T003602-3f2d2012/summary.json.
  Corrected label and guarded embedded-session navigation against unsaved/uncertain
  writes. Narrow StudyPanel run passed 63 total including runner checks, 0 failed:
  .qualification/verify/20261114T003753-86da39a7/summary.json.
- Independent review P2: FSRS recomputation alone did not replay active policy or
  daily budgets. Added shared review_journal transaction order, exact record
  coverage, active-policy and prefix-budget replay; bounded DayClock conversions.
  Added regressions for rehashed lowered budgets and future-policy rebinding.
  Focused review_ passed 69 total /0 fail/5 helper skips, audit passed:
  .qualification/verify/20261114T004247-b77cb7bd/summary.json. Follow-up static
  review of only the journal boundary is in flight.

## Next

Implement frozen subject/topic assessments with pinned composition/activity
selection and feedback withheld until all parts complete; preserve outcome-specific
coverage/unknowns and exclude instructional/unsupported items explicitly. Extend
native/portable validation and tests. Then scoped course/native/process-crash
coverage for review/assessment, final types/format, affected and full candidate
checks. Full/native/packaging-human/Sonar/CI/publication remain not_run. M4 is
incomplete and no issue is Done. No observer or GUI process.

## Integrated candidate update

Frozen subject/topic assessments implemented through typed service/native main-only
commands and UI, with immutable pin selection, reserved session IDs, global feedback
policy and separate outcome/unknown coverage. Native/portable history includes
frozen partial runs. Independent bounded assessment review found no material
linkage/admission issue. Scheduler journal follow-up found event reordering;
fixed by requiring the model-validated event order, with a rejection regression.

Scoped assessment checks passed 61 total/0 failed/5 helper skips, audit passed:
.qualification/verify/20261114T005858-7ad157d0/summary.json. Prior attempt
20261114T005609-e032fc3a failed two package captures: new exact trusted schema has
132 objects versus the old enumeration bound128; adjusted bounded enumeration to160,
retaining exact bundled DDL comparisons.

Integrated affected: 531 passed/3 failed/5 helper skips, audit passed:
.qualification/verify/20261114T010246-5fbac8d4/summary.json. All three failures
were historical fixture expectations: future history version2 is now current
(use3 and positively accept1); migration check expected8 rather than current9;
and schema7 fixture left new schema9 objects behind. Corrected fixtures and added
positive native schema8 immutable-admission/restored-copy migration coverage.

Scoped compatibility plus conflict UI passed 68 total/0 failed/5 helper skips,
audit passed: .qualification/verify/20261114T010703-3ef0d75a/summary.json.
Conflicting study writes now reload current state while retaining the local response
separately, restore only to the same pinned activity and use the new revision.
Embedded navigation retains unsaved/uncertain responses. Review-rated process-kill
before/after commit passed in the integrated affected run; no duplicate event/state.
The content-only Database test shim is cfg(test); production always chooses history
explicitly. No public API removed.

Full candidate gate is next. During execution no source/doc edits are permitted.
No native GUI, human qualification, Sonar, remote CI or publication was performed.

## First full candidate result and final corrections

Full .qualification/verify/20261114T011059-8a529120/summary.json executed
537 tests /0 failures/10 helper skips, audit passed, but the profile FAILED Clippy:
complex scope return type and an unnecessary unit-ID clone in a test. Build,
frontend coverage, format, full Rust, release GLib regression, Trivy, exact license
evidence and cargo-deny passed. Rust-doc discovery declared no tests (not_run).
Factored internal scope aliases and used a borrowed one-element slice; no lint
allowance added. Final quick passed/audit passed at
.qualification/verify/20261114T011710-93e35438/summary.json.

One UI correctness correction: per-lesson Start assessment now explicitly selects
independent supported binding pins and announces excluded worked-example/completion
or unsupported activities, rather than failing the entire mixed instructional
lesson. Native rejection of selected scaffolds remains unchanged. Added regression.
A second full run is required for this changed final candidate, not for reporting.

## Delivery and authoritative final receipt

Full candidate .qualification/verify/20261114T011935-7921d6b1/summary.json passed
538 tests/0 failures/10 helper skips with audit passed. All full-profile gates
passed; Rust-doc declared no tests (not a doctest pass). The ten helper skips are
five private child helpers excluded separately in debug/release scopes, not ten
passed tests. Sonar, remote CI, native GUI/packaged-human qualification and
publication were not performed.

A final UI-only data-integrity correction preserves extreme valid persisted
Unix-millisecond timestamps that JavaScript Date cannot represent. It displays
the original number and an explicit display-range label instead of throwing;
unknowns stay unknown and no record is rewritten. Added two boundary regressions.
Because this changes the final candidate, a final full verification follows this
receipt update with source/docs frozen. Its authoritative schema2 result is copied
to `.qualification/m4-final-checkpoint-20261113T232500Z/final-verification.json`; inspect status, audit, failures, skipped and not_run there.
The earlier 538-test full result describes only its earlier candidate.

Implementation coverage now includes all approved M4.1/KER-131 and M4.2 slices:
offline four-family study/instruction, immutable evidence/durable drafts and
corrections, pinned eligible recall schedules/clock/day policy/workloads, separate
practical reminders, frozen topic/subject assessments and distinct outcome/unknown
coverage, scoped native services, and opt-in/native recovery of new history.
Synthetic cross-course IPC and process-kill before/after commit are covered.
No actual learner profile/content, host policy or permissions configuration was
changed. No issue was marked Done; no commit, push, merge or publication.

Recovery checkpoint `.qualification/m4-final-checkpoint-20261113T232500Z` contains the final dirty/untracked source archive,
verified SHA-256 manifest, HEAD/status and staged/unstaged patches. All unrelated
baseline files remain preserved. The final JSON/raw runner paths are the evidence
for local gates; omitted release/human/service gates remain separate. Next action:
read that final receipt, do not retest unchanged passing inputs for reporting, and
await the user's next authorized step. No GUI or publication is implied.

Tools/skills used: Git, rg/bounded reads, Python/tar/SHA-256, Serena, Cargo, external
verification runner (Nextest/Vitest/Trivy/cargo-deny), Firecrawl Cloud, selected
engineering-workflow, memoquill-learning-evidence and read-docs/firecrawl guidance;
one bounded independent reviewer. No observer agent or screenshot capture.
