# M4 study foundation checkpoint

## Authority and baseline

The user approved full M4 (KER-102 + KER-131, followed by KER-103) and instructed
implementation, then explicitly requested fixing Serena. This approval remains
valid for continuation. No commit, push, merge, GUI launch or changes to existing
learner-authored content were authorized/performed. Linear was read during
preparation; no tracker mutation or completion claim was made.

Branch `dev`, HEAD `7325467b0bb43c14477624055562d3ddd415c77b`. Pre-existing dirty
work remains preserved; 61 baseline files other than the intentionally extended
handoff remain byte-identical to the original verified snapshot
`.qualification/m4-preparation-preserve-20261113T171905Z/`. Do not reset/switch
through the working tree or commit unrelated changes.

## Implemented first slice

- Schema 8 adds course-local study sessions, immutable attempts, separate judgments
  and operation receipts. Existing migration bytes remain unchanged.
- Sessions pin a committed composition/activity revision, save response drafts and
  assistance, enforce expected revisions, and atomically commit attempt/judgments/
  cursor/receipt. Identical retries return the original receipt; changed payloads
  conflict. Self-report corrections append and retain prior records.
- Native package admission validates these records, revision pins, judgment chains
  and the originating Submit operation. Session enumeration is paginated.
- Course-bound native ports/commands and the first offline study panel support
  start/resume, explicit draft saving, recorded hints/reveal, distinct self-rating
  and self-assessment, frozen assessment feedback, history and retry of an
  uncertain operation with the same UUID. No study calls occur from author preview.

Owned source paths: `src-tauri/src/study_commands.rs`,
`src-tauri/src/workspace/study/`, migration `008_study.sql`, registration in
  `workspace/{mod,database,migrations,service}.rs` (including schema-aware migration
  test expectations), native admission in
`workspace/packages/validation.rs`, command registration in `src-tauri/src/lib.rs`,
`src/study/`, and the provider/course-port/LessonTabs wiring. Owned planning/report
changes are `docs/planning/m4-execution.md`, this report and the current handoff.
Preparation documents from the preceding step also remain dirty.

## Validation and review

- Serena repaired: only project `language_servers` changed to Rust/TypeScript;
  missing rust-analyzer installed for existing toolchain, and Serena-pinned TS
  server dependencies installed in Serena's own directory. Configuration backup:
  `.qualification/serena-repair-20261113T173004Z/`. Health-check passed, and native
  MCP symbol overviews succeeded for Rust and TypeScript in the original session.
  Live activation used the existing local dashboard API without GUI/restart.
- Initial targeted study check: 6 passed, 0 failed, 4 reported helper skips;
  audit passed. Receipt: `.qualification/verify/20261113T194218-2655350e/summary.json`.
- Bounded independent read-only review found two P2 issues: listing failed at 65
  sessions, and native admission did not link attempts to Submit receipts. Both
  corrected; regressions added for pagination and rehashed attempt relabeling.
  No broad repeated review or observer agent was used.
- Quick gate passed TypeScript, Rust formatting and 56 runner self-tests; audit
  passed. Receipt: `.qualification/verify/20261113T194815-5318fc67/summary.json`.
- Broad affected check recorded 482 passed, 4 failed, 4 helper skips. Receipt:
  `.qualification/verify/20261113T194847-804b56a2/summary.json`. Failures: two
  migration tests assumed schema 7 remained latest, one new transport test used a
  cross-realm `instanceof Uint8Array`, and an existing LessonComposer interaction
  test could not find its renamed node. Corrected the migration expectations and
  transport assertion; did not modify the existing UI test or claim its original
  failure cause is established.
- Targeted correction run passed 7 existing LessonComposer tests and 3 migration
  tests (plus 56 runner self-tests), with 4 helper skips; audit passed. Receipt:
  `.qualification/verify/20261113T195133-4ca1f911/summary.json`. This CLI accepts
  only one --frontend and one --rust filter; repeated values replace earlier ones.
- Final study-specific run passed 12 tests (3 frontend + 9 Rust), 0 failed,
  4 helper skips; audit passed. Includes the two independent-review regressions
  and schema 7→8 preservation. Receipt:
  `.qualification/verify/20261113T195228-b3cd3378/summary.json`.
  The broad affected suite was not rerun after these corrections; keep its failed
  receipt distinct from the passing targeted runs. Earlier Rust compile failures
  were corrected; failed/nonexistent reports are not passes.
- A final quick attempt became stale because documentation changed while it ran;
  its audit passed but it is not validation success. Receipt:
  `.qualification/verify/20261113T195332-110d7294/summary.json`. A fresh bounded
  types/format run then passed with source/docs held unchanged until completion:
  `.qualification/verify/20261113T195428-3255b482/summary.json` (both checks and
  audit passed; no tests selected).

## Incomplete acceptance and next action

This is a partial M4 foundation, **not completed M4.1, KER-131 or M4.2**.
Remaining: complete provider-independent assessor/model provenance and uncertainty
contracts; instructional variants/bounded-cloze decisions; durable phase/time
evidence; correction UI and broader study interaction coverage; explicit opt-in
portable history and native recovery/migration compatibility for pre-M4 packages;
crash/recovery/course-scope integration and malicious-input boundaries; FSRS
version/API/license verification, recall-only scheduling, clock/DST/version cases,
workload/reminders and topic/subject assessment scope. Preserve content/history and
existing package semantics. Reuse existing learning evidence; use arXiv MCP,
Consensus/Firecrawl Cloud and research skills only for material unanswered questions.

Full milestone, native/packaged/human, security/license acceptance, CI and
publication are not_run here. KER-97 remains deferred Backlog until manual M7
Fedora/X11/Orca acceptance. Passing software tests do not prove learning efficacy.

Next: continue the approved M4 implementation in a fresh chat to avoid current
excessive context overhead. Start from this report and the current handoff, inspect
Git status, preserve dirty work, and complete the listed criteria. The existing
LessonComposer failure did not reproduce in its targeted file; inspect its
async-ready condition if it recurs rather than retrying it indefinitely. Do not
re-read old transcripts or rerun unchanged successful checks merely for reporting.

Tools/skills used: Git, rg/bounded file reads, Python, apply_patch, native Serena,
the external verification runner, a bounded independent reviewer, local Serena
CLI/API, tool-catalog, engineering-workflow, memoquill-learning-evidence,
diagnosing-bugs and mcpdo (connection inspection only). No GUI or external research.

Verified current-code backup: `.qualification/m4-study-checkpoint-20261113T175428Z/`
(67 dirty/untracked files, archive SHA-256 verification passed). Final receipt
wording in this document and HANDOFF was updated after that backup.
