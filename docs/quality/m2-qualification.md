# M2 qualification ledger

Application source: 5c72fdb on dev. M2 implementation and integration fixes are
committed; KER-99/KER-116 remain open for platform/manual acceptance. No push,
merge, remote CI or release is implied.

| Acceptance area | Current evidence | Remaining gate |
| --- | --- | --- |
| KER-109 readiness/contracts | Contracts and explicit development-only desktop deferral | Physical GNOME / human audible Orca, KER-97 |
| KER-110 organization | Domain move/reuse/archive/DAG tests; native X11 subject/collection workflow | Integrated Wayland/Orca acceptance |
| KER-111 canonical documents | Versioned codec, stable anchors, passive unsupported data; adapter/document tests; native X11 prose/math/code/IME | Integrated platform/manual acceptance |
| KER-112 persistence | Upgrade/recovery/receipt tests; native X11 restart restores documents, notes and exact pins | Integrated platform acceptance |
| KER-113 authoring | Editor/material tests and packaged X11 save/undo/redo/real IBus/reopen | Wayland/Orca; physical reflow assessment |
| KER-114 originals | Managed byte/provenance tests; packaged chooser/cancel and literal HTML preview survive restart | Integrated platform acceptance |
| KER-115 navigation/search | Domain/component tests; packaged offline search/focus/results and four subject workflows | Integrated platform/manual acceptance |
| KER-117 contracts | Unit ownership, closed registry/actions, budgets, versions/history boundary | Done for execution; live M6 remains outside M2 |
| KER-118 frontend ports | Injection/replacement/state-boundary tests; single integration review and corrected view-state ownership | Done; full M2 platform acceptance is separate |
| KER-119 compositions | Schema5/durable drafts/pins/validation/preview/apply/receipts; unknown repair and subtree undo; published-pin lifecycle regression | Done; integrated acceptance remains KER-116 |
| KER-120 constructor | Eight components/four activity families; isolation/recovery/forms/controller tests; native X11 independent tabs/save/reopen; actual-host exercise/preview/save | In Review pending integrated platform/manual acceptance |
| Preview/history boundary | Occurrence-local answers/help, pinned earlier snapshots, explicit simulation labels; no real M4 writes | Actual M4 study/history is outside M2 |

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

Unaccepted: nested Wayland WebKit self-SIGABRT at first input; its cause is not
established by adjacent graphics warnings. The saved Orca speech audit failed all
five expectations. One cold-readiness probe then timed out140 seconds before app launch;
its journey/audit were NOT RUN. The experimental candidate is retained ignored,
not shipped. This does not establish the cause of the cold-start failure.
Physical GNOME, human listening and full visual/reflow acceptance are still open.

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
