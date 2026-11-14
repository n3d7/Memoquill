# M4 evidence, recovery and portable-history continuation

## Authority and preserved work

The user instructed continuing the previously approved full M4 scope in this chat.
KER-102/KER-131 precede KER-103. No commit/push/merge, GUI launch, screenshots,
tracker mutation or changes to existing learner content were authorized/performed.
Branch dev; HEAD `7325467b0bb43c14477624055562d3ddd415c77b` remains unchanged.
All 61 baseline files other than the intentionally extended HANDOFF were verified
byte-identical against `.qualification/m4-preparation-preserve-20261113T171905Z/`.
The earlier [foundation receipt](2026-11-13-m4-study-foundation.md) remains valid
for its own code and checks; this receipt records the subsequent delta.

## Implemented

- Optional server-observed UTC Unix-millisecond times for session start, first
  acknowledged response save, submission and judgment. These are wall-clock
  observations, never measured active study time or inferred retention. Legacy
  unknowns remain unknown, including after resaving an existing legacy draft.
- Immutable session context and judgment provenance pin composition/activity
  revisions, exact definition digest and local/learner assessor identity.
  Explicit disputed outcomes and uncertainty remain separate from raw response,
  assistance and self-rating. The UI appends corrections against the latest
  self-report while retaining the original. Model/prompt/rubric identity and
  numerical uncertainty have reserved DTOs; native admission and mutations do not
  activate model assessment before M6 scoped policy.
- Native package admission now supports exact bundled schema 7 (M3) as well as
  current schema 8. It verifies schema objects/checksums/domain references before
  acceptance, preserves the staged package bytes and migrates the restored copy
  only on normal workspace open, retaining migration backup. Other historical
  package schemas remain unsupported until separately qualified.
- Portable history is explicitly opt-in, default false through UI/native/service.
  The versioned `studyHistory: 1` section includes sessions, immutable attempts,
  judgments and operation receipts using a fixed row whitelist. Original JSON
  bytes/digests and row order are retained; reconstruction still undergoes full
  native semantic validation. Undeclared history and unsupported section versions
  are rejected. Old content-only omission declarations remain readable.
  Included history contains saved study drafts and earlier draft snapshots in
  receipts; UI consent/summary explicitly distinguish these from omitted authoring
  lesson drafts. Content-only export continues excluding study history.
- Bounded cloze is recall definition version 2, with one literal `{{answer}}`
  token, no additional template tokens and an answer/response limit of 1024 bytes.
  Rendering shows `[blank]`; no template/code executes or expected answer is
  substituted. Authoring offers the variant. Version 1 definitions remain valid.
- Bounded primary-source FSRS investigation is saved in
  [Rust FSRS evidence](../research/fsrs-rust-integration.md): 7.0.0, FSRS-7,
  three-field MemoryState including stability_fast, API and numerical fixture.
  No FSRS dependency was installed or scheduler implemented. Full license text,
  actual defaults/transitive graph and compiler compatibility remain to verify.

Owned additional paths: workspace study module and lesson activity validation/tests;
packages/{archive,validation,tests,portable}.rs and portable/{records,wire}.rs;
service export plumbing; package_commands.rs; frontend lesson types/forms/rendering,
activityPrompt.ts; study panel/types/tests; package UI/port/types/tests; this receipt,
execution decisions, FSRS evidence and current handoff. Preserve unrelated dirty
runner/package/notices/tools/engineering work.

## Evidence and limitations

- Evidence-specific affected: 16 passed, 0 failed, 4 helper skips; audit passed.
  `.qualification/verify/20261113T200308-8a3347d5/summary.json`.
- Quick types/format and 56 runner self-tests passed; audit passed.
  `.qualification/verify/20261113T200339-c749bd78/summary.json`.
- Three headless native-package regressions passed, 4 helper skips; audit passed.
  `.qualification/verify/20261113T200504-53c78de4/summary.json`.
- Portable targeted run: Rust passed; UI opt-in test initially failed because its
  new fireEvent import was missing. Corrected import; targeted UI 4 passed, audit
  passed at `.qualification/verify/20261113T201016-a2ef6dfb/summary.json`.
- Broad affected before cloze: 500 passed, 0 failed, 4 helper skips; audit passed.
  `.qualification/verify/20261113T201148-a58e03a7/summary.json`.
- Cloze quick initially found a missing userEvent name in the new authoring test;
  replaced it with the already imported fireEvent. The failed receipt is
  `.qualification/verify/20261113T201524-d3f57d1d/summary.json`, not a pass.
- Final broad affected after cloze: **504 passed, 0 failed, 4 helper skips**;
  audit passed at `.qualification/verify/20261113T201547-7a7c871b/summary.json`.
  Rust-doc discovery found no tests, not a Rust-doc pass. Final quick is recorded
  separately below. No unchanged passing run was repeated solely for reporting.
- Bounded independent reviews of new metadata/native-7 and portable-history
  deltas found two P2 issues: invented first-save time for old drafts, and ambiguous
  wording about included saved study drafts. Both fixed; regression coverage
  included in the 504-test receipt. Final bounded cloze review found no material issue; it did not run tests.

M4 remains PARTIAL; KER-102/KER-131/KER-103 are not Done. Remaining: actual
instructional-phase flows beyond context labels, scoped/native-command/course
isolation and process-crash coverage; then pinned/licensed FSRS adapter, durable
versioned review events/state, recall-only eligibility, time-zone/study-day/DST/
rollback/long-gap cases, workload/reminders and frozen topic/subject assessments.
Full milestone/native GUI/packaged-human gates, CI and publication remain not_run.
KER-97 remains deferred Backlog until manual M7 Fedora/X11/Orca acceptance.

Tools/skills: Git, rg/bounded reads, Python, apply_patch, Serena, external
verification runner, engineering-workflow, memoquill-learning-evidence, bounded
independent reviewer; research/read-docs/firecrawl with a bounded research agent
and Firecrawl Cloud primary pages only. No private data sent externally; no GUI.

Final types/format quick passed with audit at
`.qualification/verify/20261113T201819-6c8466ea/summary.json` (no tests selected).
`git diff --check` passed. Product source was unchanged after these checks; only
receipt/handoff wording was finalized. All owned verification/research/review work
finished; no observer/GUI or test process is left running.

Verified continuation backup: `.qualification/m4-continuation-checkpoint-20261113T181909Z/` (archive/manifests and staged/unstaged patches).
