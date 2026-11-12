# M3 qualification ledger

Active delivery: [completion design](../superpowers/specs/2026-11-09-m3-completion-design.md)
and [plan](../superpowers/plans/2026-11-09-m3-completion.md).
Status: IN PROGRESS; KER-100/KER-101 are not complete.
Initial extraction slice is committed as `0713d74`; completion plan `a315445`;
original synthetic format fixtures `ffdea7d`. Remaining evidence is filled only
after the corresponding final tests/review run, not inferred from dispatch.

## Acceptance matrix

| Acceptance | Required evidence | Current state |
| --- | --- | --- |
| Durable jobs, mapping and item reconciliation | Real DB restart, cancel, malformed/bounds, selected partial apply tests plus UI | Task 1 scoped reviews accepted through `3f1f227`; final integrated gates pending |
| Exact retry/duplicate/unchanged reimport | Lost-response/restart receipt and canonical-count tests | Task 1 backend tests/review accepted; final integrated gates pending |
| Changed reimport preserves edits/annotations/history | Conflict previews/resolutions and immutable-revision tests | Task 1 tests/review accepted; synthetic native subset passed below |
| Guarded undo | Safe compensation and stale/dependent refusal tests | Task 1 tests/review accepted; synthetic native subset passed below |
| Rebuildable scoped FTS5 | Unicode/code/short queries, beginning/middle/end scale, rebuild and isolation tests | Task 1 tests/review accepted; native search/rebuild subset passed below |
| PDF/EPUB/DOCX/HTML and attachments | Honest capability/status matrix, originals/access/passivity/bounds fixtures and explicit PDF gate | Pending Task 3 |
| G1 native full restore | Active-WAL DB snapshot plus asset lease/barrier; current IDs/revisions/lessons/jobs/preferences/drafts/receipts roundtrip | Backend `7e6efed` tested; independent review domain-validation fix in progress |
| G2 portable declared subset | Versioned manifest/JSON/NDJSON/readable projections, omissions/remaps and source inclusion reports | Pending Task 2 |
| Untrusted archive safety | Path/symlink/duplicate/bomb/corruption/future-schema limits, failed restore leaves live courses unchanged | Pending Task 2 |
| Complete/incomplete reporting | Missing originals and unsupported sections never claim full backup/restore | Pending Task 2 |
| Future M4 history adapter | Versioned history-section contract and future-schema fixtures; no invented M4 runtime | Pending Task 2 |
| Integrated UI | Course switching, review/error/retry/cancel and keyboard-oriented interaction tests; X11 native run only with permission | Pending integration |
| Delivery | Final suites/build/security/licenses, independent review, substep commits, dev→main PR readback without merge | Pending final gates |

## Already verified baseline

Resumed verification on `47a47f7` (2026-11-10): frontend coverage passed 148 tests
in 27 files. Native library passed 199 tests with 3 existing ignored tests under
host execution. The restricted sandbox first failed 12 native tests with a
read-only filesystem error. Startup initially passed 3/4, with a NUL-environment
spawn error; all 4 passed with a clean environment. This environment-sensitive
result is recorded separately from product failures. These checks do not close
the remaining Task 1 acceptance or replace final milestone validation.

Independent review accepted the `750743d` lineage/search corrections. Review of
`47a47f7` identified a UI baseline-selection gap for review-only descendants;
fix round 2 is in progress. No full M3 approval, package qualification or release
is implied.

Before `0713d74`, frontend 145 tests/26 files, native library 182 pass/3 ignored,
startup 4 pass and 0 doctests; build/fmt/Clippy passed. Those checks were refreshed
at continuation before committing. Native original-byte/receipt preservation,
bounded passive extraction, partial/unsupported/failed outcomes and hostile-text
UI behavior were verified, including malformed CSV quote RED→GREEN. These are
baseline results, not verification of later uncommitted M3 changes.

Fixtures regenerate byte-identically; `pdfinfo original.pdf` reads one unencrypted
PDF1.4 page with no JavaScript. This does not qualify an in-app PDF parser/viewer.
No private book, dataset or learner content is included in fixtures.

Full human audible speech/reflow and supported release acceptance remain M7.
Automated DOM/IPC/native checks cannot be promoted to human accessibility evidence.
An isolated synthetic-profile screenshot was captured with the user's task-specific
permission while diagnosing the intermediate candidate below. Keep original
failed/skipped/unsupported evidence visible.

## Intermediate native candidate — 57e2ddf

An immutable Git archive was built as an RPM and installed only into the private
rootless test image. RPM SHA-256:
`b614be3dddbf25eb69f793e213237d84b6c570acac5adc53046ceaa115203cd0`.
Image ID: `8cfda80a3e6535d096f6e32cf09b0af4435c9e29b8907e2ad52172bd05dd3e90`.
Network disabled; private X11/D-Bus/profile; no host desktop sockets.

Run 1 did not reach the journey because the input omitted `ibus_control.py`.
After including the driver helpers, runs 2/3 created a subject/collection/material,
attached the original CSV with provenance and displayed its beginning/end bytes.
Creating the import preview then failed and retained an exact retry. Native
`build.rs` command declarations and `capabilities/main.json` omitted ingestion
permissions, despite registered handlers. This is a real integration gap being
fixed; these runs are **not a passed M3 journey**. Original-copy and package/recovery
checks were explicitly not run. Run 3 includes a private synthetic failure PNG
and AT-SPI controls. Local ignored evidence is under
`.qualification/m3-57e2ddf/results*`; final qualification must use the fixed candidate.

## Later native subsets — 3f1f227

The command/permission integration gap above was corrected and independently
reviewed. Immutable candidate RPM SHA-256:
`8872e88b2f9fc82518c612faa9976067d74b8856ea15e1a044aea494c3ef69cf`;
installed image ID:
`74d34e847e6030ab348078dd0adce63e39652994891b2fab560e7ab58de66fd4`.
The same private network-disabled X11/D-Bus/profile isolation was retained.

`.qualification/m3-3f1f227/results-run4` passed the CSV ingestion/search/rebuild
subset and changed-source three-way comparison, explicit human-edit approval,
partial reimport, compensating undo restoring human text, parent preservation and
unsafe initial-undo refusal. That run later failed in the JSON harness: it checked
an existing unresolved status before review acknowledgement. It is not a whole
journey PASS. Earlier native-combo opening and unsupported synthetic Unicode
typing were harness issues; source-file Unicode remained covered.

After waiting for record-review acknowledgement and actual checkbox states,
`.qualification/m3-3f1f227/results-json/m3-journey.json` has no failure entry:
two named JSON materials applied and the duplicate third record remained visibly
unresolved, reconciling every input record. This is a focused JSON subset only.
Original-copy audit was explicitly disabled because that action was not yet
implemented. Packages/recovery were not exercised in either subset.

New native runs keep screenshot capture disabled under the current agent guide.
Existing synthetic captures remain historical diagnostics; no human speech/reflow
acceptance or complete M3 qualification is inferred from these subset results.
