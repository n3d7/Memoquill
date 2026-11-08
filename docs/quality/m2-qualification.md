# M2 qualification ledger

Current exit, 2026-11-08: the user resumed M2 development after the tool-catalog
refresh. [ADR-10](../architecture/decisions.md#adr-10--x11-first-core-and-deferred-platform-adapters)
keeps M1–M8 Linux/X11-only with [portable contracts](../architecture/platform-portability.md).
The current X11 functional/integrity/security exit is qualified; the final bounded
review found no blocking issues. Tracker closure is recorded in the current report. Full supported-X11 AT/reflow stays
M7; KER-97 manual evidence remains deferred/unaccepted. Other-platform runtime
work stays inactive after M8. Full visual-design timing will be discussed with the
user after M2; no redesign has been added to this scope.

Source baseline5e245ba plus the labelled-control CSS fix and qualification/test
changes below, on dev. After M2, prepare dev for main; do not start M3 or actually
push/merge/create a PR/release.

| Acceptance area | Current evidence | Exit / retained boundary |
| --- | --- | --- |
| KER-109 readiness/contracts | Contracts and explicit development-only desktop deferral | KER-97 manual follow-up deferred; no non-X11 main-development gate |
| KER-110 organization | Domain move/reuse/archive/DAG tests; native X11 subject/collection workflow | X11 exit qualified; release-level X11 AT remains M7 |
| KER-111 canonical documents | Versioned codec, stable anchors, passive unsupported data; adapter/document tests; native X11 prose/math/code/IME | X11 exit qualified; release AT/reflow remains M7 |
| KER-112 persistence | Upgrade/recovery/receipt tests; native X11 restart restores documents, notes and exact pins | X11 exit qualified |
| KER-113 authoring | Editor/material tests and packaged X11 save/undo/redo/real IBus/reopen | Functional X11 exit qualified; full AT/reflow remains M7 |
| KER-114 originals | Managed byte/provenance tests; packaged chooser/cancel and literal HTML preview survive restart | X11 exit qualified |
| KER-115 navigation/search | Domain/component tests; packaged offline search/focus/results and four subject workflows | X11 exit qualified; release AT/reflow remains M7 |
| KER-117 contracts | Unit ownership, closed registry/actions, budgets, versions/history boundary | Done for execution; live M6 remains outside M2 |
| KER-118 frontend ports | Injection/replacement/state-boundary tests; single integration review and corrected view-state ownership | Done; X11 integration qualified |
| KER-119 compositions | Schema5/durable drafts/pins/validation/preview/apply/receipts; unknown repair and subtree undo; published-pin lifecycle regression | Done; integrated X11 acceptance qualified |
| KER-120 constructor | Eight components/four activity families; isolation/recovery/forms/controller tests; native X11 independent tabs/save/reopen; actual-host exercise/preview/save | Done for functional X11 scope |
| Preview/history boundary | Occurrence-local answers/help, pinned earlier snapshots, explicit simulation labels; no real M4 writes | Actual M4 study/history is outside M2 |

## Final X11 exit evidence — 2026-11-07 runs, recorded 2026-11-08

- Final frontend coverage suite:120 tests/23 files passed. A prior119/120 run
  exposed an asynchronous preference-test assertion: the selected appearance can
  settle before the root font-size effect. The test now waits for the same125%
  condition;6 targeted preference tests and the final full suite pass. Production
  preference behavior is unchanged.
- TypeScript/Vite, Rust release and RPM build passed. Candidate RPM SHA256:
  `5c0adfe50a553e15675cb87ecbd7e5525c691d3a176973e269fd984c1d96a394`.
  Retained candidate/source hashes: `.qualification/m2-x11-exit-20261107/final-candidate/candidate.json`.
- Fresh full isolated packaged X11 journey:68 assertions passed, plus one IBus
  engine metadata record; exit0. It covers authoring, code/math/real IME,
  originals, search, all constructor families/components, isolated tabs and
  durable save/restart. Restart uses the same extracted candidate through PATH.
- The earlier attribute-query crash also reproduces on X11: a valid request for
  the empty control embedded in its label aborts WebKitWebProcess. Block layout
  for labelled text inputs/textarea avoids that inline run while retaining the
  implicit label/name. The same updated native probe fails on the old RPM and
  passes on the candidate. This does not fix the WebKit engine or establish full
  screen-reader conformance; checkbox/radio/hidden controls are excluded.
- UI-only qualification is explicitly guarded to X11+M2. AT-SPI remains enabled;
  Orca/speech assertions were NOT RUN and the skip JSON says so. Zero screenshots
  were captured. Historical speech failures remain failures, not overwritten.
- Unchanged Rust/startup/fmt/Clippy, optimized glib, prior integration review and
  scoped security evidence below are reused under the explicit reduced-check
  instruction. No dependencies/locks or Rust source changed in this exit fix.
- One bounded independent review covers the previously unreviewed source tail
  after1c1fb43, rather than repeating the whole-change review: no Critical,
  Important or confirmed Minor issues. Review did not rerun tests or assess full
  speech, other platforms, M3 or the already-reviewed baseline.

Current [exit report](../task-reports/2026-11-08-105345Z-m2-x11-exit.md).
Logs, red/green probes, final candidate and native JSON are retained under
`.qualification/m2-x11-exit-20261107/`. The first block-layout probe stopped on
an overstrict harness assumption about the label representation; it did not
crash the application. The corrected probe accepts a separate named control and
still reproduces the old-candidate failure.

## Retained earlier evidence and limitations

Passed automated gates:118 frontend tests/23 files with coverage;157 Rust library tests
(3 ignored);4 Linux startup bin tests;TypeScript/build, affected fmt/Clippy and RPM
build. Final fmt and Clippy all-targets checks also passed on5c72fdb. Unchanged
optimized glib regression was reused from7742433 (1 passed).
The single integration review reused the reviewed backend baseline and found two
Important issues, fixed in3b1c5a4/1c1fb43 with focused regressions. No second broad review.
Logs/reports: .superpowers/sdd/2026-11-04-m2-composition/, especially logs/final/.

Native X11 on1c1fb43:60 main checks plus9 reopen checks passed across retained runs.
Test selection/AT-SPI button roles were corrected; initial failed runs are preserved.
Actual host on5c72fdb: default launch without an exported renderer flag was confirmed
by the user. Native UI exercise editing, exact-key preview, two hints, tab isolation
and durable save/published-answer semantics passed. The saved exercise/answer and100%
text scale subsequently survived an actual host process restart.

Further actual-host constructor acceptance on the rebuilt focus-fix candidate:

- All four activity families were authored and exercised. Single choice distinguishes
  wrong/right keys; constructed/procedure guidance remains self assessment.
- All eight components were exercised. Canonical material and passive original
  provenance render in columns; a named section action and one-hop status agree.
- Cross-tab prompt/form/answer/hint/checklist isolation, earlier pinned content,
  durable save and process restart passed. Transient preview answers reset at restart.
- A native keyboard defect was reproduced and fixed: Close original preview now
  restores its exact lesson opener, with a visible lesson-heading fallback. The
  rebuilt RPM payload passed the original native case;47 affected tests/8 files,
  TypeScript/Vite and RPM build passed. Unchanged earlier broad gates are reused.
- This candidate RPM SHA256 is d2c55cf523f97c5728ce63c9d673f98526ceb7f2e5bd2326236949ff5a12b9db;
  its payload/source hashes are in focus-fix-candidate/candidate.json. The earlier
  linux-startup-candidate remains retained separately.
- Revisited select options were sometimes missing from AT-SPI; explicit Space and
  ordinary keyboard selection were checked against rendered/durable outcomes.
  This is not a claim of complete screen-reader support or visual acceptance.

See [native constructor report](../task-reports/2026-11-06-185702Z-m2-native-constructor.md).
M1.1 desktop setup/physical speech acceptance is explicitly user-deferred; no host install.

Deferred Wayland evidence: WebKit self-SIGABRT is now narrowed to a text-attribute
query on the application label; the [saved diagnosis](wayland-deferred-diagnostic.md)
records its stack, controls and limits. No upstream engine fix or application-independent
reproduction was established in that earlier slice; the common X11 case now has
the layout workaround above; adjacent graphics warnings are not proof of cause. The saved Orca speech audit failed all
five expectations. One cold-readiness probe then timed out140 seconds before app launch;
its journey/audit were NOT RUN. The experimental candidate is retained ignored,
not shipped. This does not establish the cause of the cold-start failure.
Historical physical GNOME/Wayland evidence remains unaccepted and is outside the
main development target. Full supported-X11 listening/reflow acceptance remains a
M7 release concern; no failed or unrun check becomes a pass.

Linux/NVIDIA startup 5c72fdb fixes a locally reproduced GBM white window via early,
process-local re-exec; explicit environment overrides remain intact. Potential
rendering performance cost is documented in the
[upstream Tauri issue](https://github.com/tauri-apps/tauri/issues/9304).
SOFA posta3583403-e8b6-42b6-909d-02e299b9df98/replybe8d9ec2-7051-47f5-8df3-cf1c4882db51
supplied supporting environment-timing advice; its Delphi API differs from Tauri.
Local reproduction, not its trust score, established applicability.

Security: Semgrep 1.183.0 checked52changed TS/JS files (two narrow sink rules) and
two post-fix frontend files:0 findings/errors;5positive/1negative fixture passed.
Trivy 0.75.0 secret scans:79 changed files,4fix files and2startup files,0 findings. Manifests/locks
are unchanged; no duplicate advisory/license collection. Full Rust SAST and
vulnerability freedom are not established. No private source/data was uploaded.

Artifacts are retained under .qualification/m2-composition/. RPM/source digests
and bundle-marker verification are in linux-startup-candidate/candidate.json.
No screenshots were captured by agents; visual observations use user-provided images.
