# M4 PR review, correction and merge gate

2026-11-14. The user authorized review/merge of PR #7 into main when ready,
Linear state updates, and main-branch protection after merge. Existing dirty dev
work remains preserved; no personal learning profile or host policy was changed.

## Independent review and correction

The `code-review` workflow was bounded to relevant risk seams rather than loading
the full 610 KB patch. Standards review of native commands, scoped admission,
ACLs and migration execution found no blocking violation. Spec review against
live KER-102/KER-131/KER-103 found one genuine missing acceptance criterion:
deliberately flawed examples lacked explicit labels, correction and review
provenance. Existing green tests did not satisfy that omitted requirement.

The correction adds opt-in constructed-response version 3 (`errorRepair`) with
closed, bounded, nonblank material/label/expectedCorrection and review
reviewer/reference fields. Versions 1/2 retain their behavior. Definitions remain
immutable revisions; generic native/portable admission and saved history retain
the complete definition. Review details are author-supplied, never factual
verification. Four-subject fixtures exercise the same declarative contract.

Both preview and study label the material intentionally flawed. Study keeps the
correction hidden until explicit reveal or feedback after submission. Showing a
flawed problem does not itself mark the expected solution revealed. New sessions
start with solutionRevealed false; explicit reveal records assistance. Version 3
remains excluded from independent assessments along with version 2 instructional
scaffolds. No semantic grader, code execution, provider, dependency or schema
migration was added. A follow-up independent review found the reported gap
resolved and no concrete regression at these seams; this is a bounded static
review, not a claim of complete re-review or independent test execution.

## Verification receipts

- Two native regressions first failed against the missing contract, then passed
  with the form checks: focused 21 passed / 0 failed / 5 helper skips, audit passed.
- StudyPanel focused: 12 passed / 0 failed / no skips, audit passed.
- Type checking passed after preserving the discriminated union when updating
  instructional material. Prior type failure evidence is retained; skipped full
  gates are not counted as passed.
- New installed RPM/private Podman X11 candidate: 12 GUI assertions passed for
  authoring, visible label/review disclaimer, hidden correction in the study
  region, explicit reveal, durable submission and reopened application readiness.
  Orderly close exited 0 and released both locks. No screenshots or workstation
  interaction occurred. See [GUI receipts](../quality/m4-gui-evidence.json).
- The first native negative check searched the whole authoring window and failed
  because the author's correction field was visible. Retained scope evidence
  confirms it was absent from the study region. The corrected region-scoped probe
  passed; that failed harness attempt is not counted as a product test pass.

The first corrected-candidate full run recorded 543 passed / 1 failed / 10 helper
skips, audit passed. Its sole failure was the old catalog-count assertion (six
variants before adding constructed v3). The exact identities already included v3;
the count and required v3 schema/provenance assertions are now updated. Focused
catalog correction passed 1 test / 0 failures / 5 helper skips with audit passed.
Only test code changed after native qualification; the shipped product is
unchanged, so packaging/native successful checks are retained without repetition.
The final full gate is repeated for this corrected test candidate.

Final full-profile gate is mandatory before merge. Its authoritative schema-2
result/raw report is copied to the stable local path
`.qualification/m4-merge-candidate-20261114/final-verification.json` after this
candidate freeze. Read status, audit, failures, skipped and not_run; this document
alone does not assert the future run passed. Do not rerun unchanged passing gates
solely to fill in this receipt. Product hashes are retained in the same checkpoint.
The earlier 540/0 result belongs to the previous product candidate.

Remote CI has no configured workflow/status checks; main had no protection before
this operation. Absence of CI is not a green CI result. Merge uses the exact
reviewed head guard. After merge, configure main for PR-only changes, no force
push/deletion and resolved conversations, including administrators. With the
single-owner workflow, no external approving review or nonexistent CI check is
required. Human Orca/reflow remains M7, deferred rather than waived. Original
workstation AlreadyOpen root cause is not asserted fixed; actual guarded native
close/reopen of this candidate passed independently.

Local evidence: `.qualification/m4-pr-merge-review/` and
`.qualification/m4-error-repair-native/`. The original dirty dev tree is retained;
PR worktree `.qualification/m4-pr-finalize` contains only the authorized M4 work.
No unrelated environment/verification-runner changes are published. Final remote
merge/protection and tracker readback will be recorded separately in the handoff
and Linear, after actual success.

Tools/skills: RTK, Git/GitHub connector/gh, Linear connector, Serena, external
verification runner, rootless Podman/X11/AT-SPI, `code-review` and
`memoquill-learning-evidence`. Primary agent owned implementation; independent
agents performed only the bounded reviews described above.
