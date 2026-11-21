# M5 startup, reflow and recovery boundaries

Candidate based on merged main `14fc1829168b071b212b1cf866b6a77cc357f75f`,
branch `feat/m5-remaining-qualification`. These are local results from the
verification runner. The input used the existing synthetic 10 GB of originals
with 10,000 resources and 1,000,000 attempts.

- Due overview shares immutable lesson registry descriptors and borrows their
  schemas. On the same v10 fixture, it took 0.968–0.978 s versus 18.04–18.07 s
  with uncontrolled filesystem caches. The earlier exact installed candidate
  passed 19 checks, including Orca speech requests at three catalog positions.
- Startup still checks every registered original's length and SHA-256. The
  workspace owner joins up to four read-only workers before proceeding.
  Paired release observations were 10.690 and 4.770 s with one worker and
  1.203 and 1.208 s with four. Cache state was uncontrolled. Missing and
  same-size corrupt originals are covered by a focused worker regression.
- Final installed candidate reached a useful view in 3.155 s with uncontrolled
  caches and in 2.418 and 2.380 s on immediate warm restarts. Both warm samples
  meet the provisional 3 s target. Course study controls became ready 5.306,
  1.310 and 1.297 s after those useful views. Each of the three instances exited
  normally and released both locks (12/0).
- Native v1 capture rejects large DB files, asset counts and expanded payloads
  before temporary backup. Capture compares the registered assets with the
  SQLite snapshot before use. Over-budget DB rejection took approximately
  0.05 ms and created no staging in the full-fixture test. The frontend clears
  stale export success and explains that the original workspace remains the
  user's recovery copy. The installed offline path passed 8 checks, including
  the warning, absence of a save dialog and continued catalog access.
- The installed narrow view at 200% and 800 pixels passed 9 keyboard, reflow
  and lifecycle assertions. The combined first run failed on the independent
  export step and is retained. The corrected native check read the package
  region directly and allowed the asynchronous action to finish.

Final Rust checks passed 324/0 with 12 helper skips, including format and
Clippy. The focused package panel suite passed 5/0. These and the targeted
scale runs passed their audits. Receipts and raw artifacts are under
`.qualification/m5-optimized-scale/`:

| Receipt | Scope |
| --- | --- |
| `registry-cache-and-measure.json`, `registry-cache-native.json` | Due and Orca speech request observations |
| `parallel-recovery-rust.json`, `recovery-hashing-cli.json` | Rust gates and full-original hash timing |
| `parallel-recovery-native.json` | Three installed launches and lock release |
| `capture-preflight-scale-cli.json`, `capture-preflight-frontend.json` | Early refusal and user feedback |
| `narrow-and-export-native.json`, `oversized-export-native-region.json` | Narrow view result and corrected oversized export |

Native checks ran in private offline rootless Podman X11 without screenshots.
Human audible Orca playback, formative observations and full-scale recovery
are still pending. See the [manual protocol](../quality/m5-human-evaluation.md)
and the [large recovery design](../architecture/m5-large-recovery.md).

## Offline history and theme follow-up

Lesson theme scopes now supply a complete palette to their text, fields,
actions, links and focus indicators. The WebKitGTK fixture exercised keyboard
focus on six controls in light lessons inside dark courses and the reverse.
Text contrast was 7.20–14.54:1 and outline contrast was 7.20–9.38:1.
Before the change, field/button text contrast was 1.10–1.12:1. The fixture
passed 6/0, the complete frontend suite passed 245/0, and both audits passed.
The fixture sampled computed styles and keyboard focus in WebKitGTK.
The earlier native attempt passed packaging and image attestation, then
hit the WebKit title length cap when reporting samples. The correction
sent one sample at a time and reused the identical RPM.

The offline ingestion run verified CSV mapping, reimport, undo, JSON mapping
and original bytes through PDF/EPUB. Its overall report remains failed
(71 passing assertions) because the staged inputs omitted the later DOCX
fixture. The individual recorded assertions are retained.

A procedural lesson recorded one checked step, an unchecked second step,
an observation and a learner judgment. The offline progress inspector showed
the procedure as practical self-report. A second session saved a draft.
The installed RPM wrote both a complete native archive and an opted-in
portable history archive. The initial combined journey hit harness
synchronization and view-selection errors. The isolated import-only check
reused those emitted archives in a fresh offline profile, opened each restored
course, and verified the exact draft, completed/unchecked steps, original
observation and self-report evidence in both. It passed 21/0 with audit
passed. It used RPM SHA-256
`76beaac9610951a4e60ee6c6276edddf58b7044606dc727b82d1ca1ba24e033d`.

Additional runner receipts in `.qualification/m5-optimized-scale/`:

| Receipt | Scope |
| --- | --- |
| `theme-contrast-corrected.json`, `theme-contrast-frontend.json` | Mixed-theme WebKitGTK and frontend |
| `ingestion-and-recovery-native.json` | Individual ingestion assertions before the missing DOCX input |
| `practical-history-open-unit.json`, `practical-restore-native.json` | Exported fixtures and passing fresh-profile history recovery |
| `recovery-footprint-cli.json`, `recovery-footprint.json` | Read-only v10 footprint probe, execution and audit passed; no tests counted |

The retained v10 fixture has 21,932,274,688 payload bytes and 5,701,679 rows.
The largest table, `study_operations`, has 2,100,000 rows. The v1 DB, total
payload, entry count and history limits each prevent a full archive of this
fixture. Keep the early v1 refusal and its user-visible explanation. The
[large recovery design](../architecture/m5-large-recovery.md) records
the streaming path, disk and row bounds and full-fixture restoration gate.

## Full streaming recovery

Native v2 uses a sequential raw format with exact lengths and SHA-256.
The desktop takes this path when native v1 capture exceeds its budget.
The large reader applies independent manifest, payload, row and SQLite
time limits. Disk preflight runs at capture, output, inspection and
activation. The review stays course-bound and the catalogue intent binds
the restore retry to its operation identifier, title and package identity.

A read-only baseline validated integrity and canonical data for the retained
11.93 GB SQLite snapshot in 189.069 s. The full backend roundtrip took
287.423 s. It encoded the snapshot and originals in 29.655 s, verified the
stream in 217.858 s, then restored and rechecked the root in 39.305 s.
It compared the original snapshot and restored hashes, manifest, catalogue
selection and exact retry. Peak observed temporary space was about 62 GiB.
The measurement used the existing 10 GB original set without generating
additional fixture data.

The final installed RPM published a 21,934,076,358-byte stream through the
native save picker. The combined desktop journey hit its 640 s limit after
root publication and lacks a final assertion receipt. The follow-up used
the same exported archive read-only on the same RPM in a fresh private
offline profile. Inspection took 263.278 s, and confirmation plus useful
view took 35.993 s. It passed 11/0, including 10,000 visible material count,
initial material availability, release of staging after acknowledgement,
exit zero and release of catalogue and course locks. The test-owned archive
and workspace were cleaned. The installed RPM SHA-256 was
`6ebafe14b08dc31a5b1dbec88ebdb0c72d44f0e86c35e9568b201ec88cba65e3`.

Final focused package Rust passed 41/0 with 14 helper skips. Package UI
passed 6/0. Format and all-target Clippy passed. The focused offline DOCX
journey passed 10/0, including passive preview and verified original-byte
copy, addressing the earlier missing fixture. These reports passed audit.
No screenshots or host policy changes were needed.

| Receipt under `.qualification/m5-stream/` | Scope |
| --- | --- |
| `admission-corrected.json` | Package compatibility and baseline semantic validation |
| `full-roundtrip.json` | Full backend payload and exact retry |
| `desktop-package-publication.json`, `desktop-package-frontend.json`, `quality.json` | Final focused package and source gates |
| `installed-roundtrip.json`, `installed-export-ready.json` | RPM/image attestation and combined journey evidence |
| `installed-import.json` | Fresh-profile full import on the exact same RPM |
| `docx-native.json` | Missing offline ingestion check |

The follow-up remains local on `feat/m5-remaining-qualification`.
Manual Orca listening and recruited participant sessions are deferred.

## Personal-use closeout

The M5 personal-use scope covers the ordinary offline study workflow on Fedora
X11. It includes evidence and revision history, keyboard navigation, resumable
sessions, personalization and local recovery. Manual Orca playback, recruited
participant sessions and public release preparation are deferred by the user.
Previous automated Orca presentation evidence remains separate from listening.

| Requirement | Evidence |
| --- | --- |
| Progress and session continuity | PR #8 and follow-up projection/history checks, installed Resume/Due 24/0, native and portable procedural history recovery 21/0 |
| Settings and navigation | Subject and shortcut customization 26/0, narrow reflow 9/0, lesson theme contrast 6/0, large keyboard search and archive/restore |
| Local source ownership | Offline source import checks plus DOCX original preservation 10/0 |
| Full local recovery | Synthetic 21.93 GB v2 package, backend payload hashes and durable retry, installed import 11/0 |

The final broad runner passed 576 tests with zero failures and 28 ignored
helpers. Build, formatting, Clippy, dependency scan and license policy passed.
Doctest discovery found no declared doctests. The full result predates the
interruption cleanup correction and remains the broad baseline. The focused
recheck after the correction passed Rust package tests 42/0 with 14 helpers
skipped, formatting and all-target Clippy. Both runs passed audit.

Independent review identified interrupted imports and exports as a source of
orphaned package data. On reopen the exclusive catalogue owner now reclaims
only private package staging and unpublished restore staging with known names.
Published course roots, retry markers, adjacent files and symbolic links are
preserved. The follow-up review found no blocking issue.

The corrected installed RPM passed 6/0 with audit passed in private
offline Podman X11. It opened after the synthetic interruption, reclaimed
the expected directories, preserved an adjacent file, and released all
owner locks on exit. This check used small staging fixtures. The earlier
21.93 GB roundtrip was reused as evidence for the unchanged stream payload
and validation. The corrected RPM SHA-256 is
`71c679af661b111803181e24bbba260ce535612c6fabf6b53a32a40269dfd6ab`.

Linear KER-104 is Done for this scope.

Evidence lives in `.qualification/m5-closeout/`; earlier native receipts are
listed above. This follow-up is local; the earlier PR #8 remains the integrated
baseline. M6 was not started.
