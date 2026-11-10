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
| Durable jobs, mapping and item reconciliation | Real DB restart, cancel, malformed/bounds, selected partial apply tests plus UI | Pending Task 1 |
| Exact retry/duplicate/unchanged reimport | Lost-response/restart receipt and canonical-count tests | Pending Task 1 |
| Changed reimport preserves edits/annotations/history | Conflict previews/resolutions and immutable-revision tests | Pending Task 1 |
| Guarded undo | Safe compensation and stale/dependent refusal tests | Pending Task 1 |
| Rebuildable scoped FTS5 | Unicode/code/short queries, beginning/middle/end scale, rebuild and isolation tests | Pending Task 1 |
| PDF/EPUB/DOCX/HTML and attachments | Honest capability/status matrix, originals/access/passivity/bounds fixtures and explicit PDF gate | Pending Task 3 |
| G1 native full restore | Active-WAL DB snapshot plus asset lease/barrier; current IDs/revisions/lessons/jobs/preferences/drafts/receipts roundtrip | Pending Task 2 |
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
No screenshots captured. Keep original failed/skipped/unsupported evidence visible.
