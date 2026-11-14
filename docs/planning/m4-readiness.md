# M4 readiness and proposed scope

Update: the user approved this scope and instructed implementation on 2026-11-13.
The preparation receipt below is historical. Current implementation and remaining
criteria are in [M4 execution](m4-execution.md) and the current handoff.

2026-11-13. Planning only; application implementation requires user agreement.
Russian conversation; English artifacts/tracker. Linux/X11 remains the current
platform target. No GUI, learner-content edits, external publication or tracker
mutation is authorized by this preparation.

## Preserved baseline and evidence

- Branch `dev`, HEAD `7325467b0bb43c14477624055562d3ddd415c77b`.
- Before edits, preserved 62 dirty/untracked and local instruction/skill files in
  `.qualification/m4-preparation-preserve-20261113T171905Z/`. The snapshot includes
  a SHA-256 manifest, verified archive, Git status and staged/unstaged binary
  patches. It is local and ignored. Existing work remains in place, uncommitted.
- Existing dirty work: `.gitignore`, `docs/HANDOFF.md`, third-party notices,
  the M3 completion report, `package.json`, engineering docs, earlier reports,
  `tests/qualification/VerifyInstalled.Containerfile` and `tools/`.
- M3 PR #5 and cleanup PR #6 merge receipts are in the current handoff and supplied
  by the user. This preparation did not query GitHub or refresh remote refs.
- Live Linear project inventory returned one complete page. KER-99, KER-117–120,
  KER-127–130, KER-100 and KER-101 are Done. KER-102, KER-103 and KER-131 are Backlog.
  KER-131 is the sole child of KER-102. No tracker state changed.
- Source prerequisites exist: versioned activity schemas and four families in
  `src-tauri/src/workspace/lessons/catalog.rs`, revision records in
  `src-tauri/src/workspace/lessons/types.rs`, authoring interaction fixtures in
  `src/lessons/forms.test.tsx`, per-course roots/services in
  `src-tauri/src/workspace/courses.rs`, and registered migrations in
  `src-tauri/src/workspace/migrations.rs`. These are inspected contracts, not new
  runtime test passes or completed M4 study behavior. No FSRS dependency appears
  in the current Rust manifest/lockfile.

## Dependency and acceptance reconciliation

| Work | Live dependency | Proposed completion criteria |
| --- | --- | --- |
| [KER-102 / M4.1](https://linear.app/kerigan/issue/KER-102) | KER-99 Done; reuse KER-117–120 and course boundaries | Offline creation/completion of recall/bounded cloze, single-choice, constructed-response/problem and procedure activities; immutable prompt/answer/rubric/composition revisions; feedback/hints and explicit reveal state; durable drafts/sessions; atomic idempotent attempt/evidence/session advancement; unknown future types remain inert; versioned ownership/recovery integration without deleting history. |
| [KER-131](https://linear.app/kerigan/issue/KER-131), within M4.1 | Child of KER-102; settle evidence contracts before attempt persistence | Keep raw responses, observed/declared/unknown help, self-ratings and deterministic/self/model judgments distinct. Record assessor/version/provenance/uncertainty and append corrections. Assisted performance or an AI opinion never silently becomes independent competence. M4 supplies contracts; live external assessment remains M6. |
| [KER-103 / M4.2](https://linear.app/kerigan/issue/KER-103) | KER-102 Backlog | Pinned FSRS adapter/defaults for eligible repeatable recall only; durable review events/state and algorithm/parameter/generation versions; deterministic timezone/study-day/DST/rollback/long-gap/overdue behavior; workload controls and separate fixed/manual practical reminders; frozen assessment scope/revisions/feedback policy and outcome-specific coverage/unknowns. Preserve history across content, parameter and budget changes. |

M3 and M4 are parallel-capable successors of M2 in the roadmap; M3 is not a formal
Linear blocker of M4. M3.2 is now available for mandatory new-history package and
recovery integration. M5 joins M3/M4; KER-103 also blocks M7/KER-108 in Linear.
KER-97 remains Backlog and manual acceptance is deferred to M7, not waived.
Its old description/edge still refers to blocking already-Done KER-98; treat this
as historical tracker wording, not a new M4 development blocker. No edges changed.

## Proposed implementation order after approval

1. Freeze the M4.1/KER-131 contract delta against existing activity/revision and
   course services: attempt identity, retry conflicts, evidence/judgment separation,
   draft/session transitions, historical relevance and recovery format changes.
   Specify bounded cloze and reusable instructional phases within the four
   families; author preview must not create study evidence.
2. Deliver M4.1 as offline vertical slices: migrations and typed services, then
   learner-paced study and authoring/feedback interaction. Include interrupted
   processes, acknowledged-attempt durability, course isolation and package/restore
   round trips before accepting the stage. Preserve existing learner content.
3. Deliver M4.2: verify the selected FSRS release/API/license/toolchain against
   primary documentation, pin it, define clock/event fixtures, then add eligible
   recall queues, workload/reminder controls and scoped assessments. Keep policy
   independent of widgets/window lifetime and clocks behind explicit adapters.
4. Qualify the integrated candidate with focused regressions followed by `full`;
   inspect independent evidence for recovery, authorization and public contracts.
   Request GUI authorization only when an isolated native journey is concrete.

Full M4 means KER-102 + KER-131 + KER-103. Recommended sequencing approves this
scope while delivering M4.1 before M4.2. No built-in semantic/AI grader, optimizer,
code execution, executable activity plugins, live MCP assessment, gamification,
full visual redesign or additional platform implementation is included.

## Verification and remaining decisions

Required test families are T01/T05/T08/T10 plus T02/T03/T04 for new persisted
history, migrations and ownership/recovery. Cover four-subject English/Python/
mathematics/cooking fixtures, inert unknown types, malicious render content,
preview/evidence separation, cross-course denial, same-ID replay/conflicting retry,
crash/draft/resume, immutable revisions/help/judgments, and old/new package versions.
Scheduler tests add golden/property cases, invalid ratings, event/state atomicity,
no-history defaults, clock boundaries and explicit schedule generations. Practical
self-report and choice recognition must retain their evidential limits.

Use `python3 tools/verification/verify.py run --profile affected` during edits,
`--frontend`/`--rust`/`--only` for concrete regressions and `full` on the final
candidate. Await final JSON; read detailed logs only for named failures. Native
qualification requires separate explicit authorization and isolated synthetic data;
physical/human KER-97 remains M7. Existing report `audit` is not a new test pass.
Do not repeat unchanged passing checks or launch observer agents/mqwatch.

This preparation changes documentation only. Product tests, full/native gates,
CI and publication are **not_run** here. Final document checks and preservation
results are recorded in the linked preparation report.

Existing learning evidence is reused: `docs/product/learning-model.md`,
`docs/research/learning-science.md`, `docs/research/learning-activities.md` and
`docs/planning/learning-productivity-followup.md`. No new efficacy claim is made.
If a material uncertainty needs research, use arXiv MCP, Consensus and Firecrawl
Cloud with primary-source verification; never send private learner data.
FSRS API/version details and the precise storage/UI design remain implementation
decisions after scope approval, not completed research or frozen architecture.

Next action: obtain agreement on full M4 scope and the staged order above.
No M4 implementation has started.
