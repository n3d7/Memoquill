# M1.1 — Fedora desktop qualification candidate

Outcome: **ready for user review; manual desktop/Orca acceptance remains**. [KER-97](https://linear.app/kerigan/issue/KER-97/m11-qualify-the-fedora-desktop-and-accessibility-baseline); M1.2/KER-98 has not started. KER-97 was moved to **In Review** and read back with unchanged project/milestone/priority/dependency; KER-98 remains Backlog. This is a qualification fixture, not the complete learning application or a public release.

The pinned Rust/Tauri 2 + React/TypeScript shell now builds a real Fedora RPM. It supports ephemeral generic prose, Unicode, bounded mathematics, passive code, strict typed validation, preview, native name-only file selection, accessible operation results, themes and text scaling. English-only UI remains separate from arbitrary-language learning materials. No database, real importer, learning engine or MCP adapter was added.

Material fixes address plain-text paste before HTML parsing, native modal Tab containment, picker focus after React commit, and explicit result reading/return focus. Important feedback has a dialog and **Read last result**, rather than relying solely on polite notification behavior. Actual optimized glib string iteration crashed with the registry dependency; an inspected immutable two-line upstream-fix backport and release regression mitigate that exact defect (ADR-08).

Validated on the final candidate:

- `npm run check`: build/types, 14 UI tests/axe, Rust fmt/Clippy, 11 Rust tests and a targeted optimized dependency regression pass.
- Actual offline-installed RPM: **35 native interaction checks and 7 Orca presentation checks in each of X11 and native Wayland**. Covers success/error/read-result flows, keyboard recovery, source reading, picker cancel/select, dark/200% settings and real private IBus composition in input/contenteditable.
- X11 app/child network tracing observes no internet socket creation/outbound address calls. Host SELinux remains Enforcing; private test profile limitations are documented.
- CRAP maximum 8, no Dryer candidates >=0.8; TypeScript 40 mutation sites killed and corrected unchanged Rust evidence 5 killed, zero survivors/uncovered sites reported by those operators.
- Semgrep 1.182.1: 26 authored files, zero findings/errors. Trivy 0.75.0: no High/Critical or secrets; one version-visible mitigated glib Medium and three Low no-healthcheck notices on one-shot images. RustSec's proc-macro-error maintenance warning remains.
- Dependency-Track 5.2.0 completed the dedicated Memoquill SBOM, 608 components, zero counters. Detailed findings are unavailable with the existing key (403); unchanged lock hashes match that upload. Counters do not override the independent glib finding.

The audit itself needed correction: Orca buffers its log, so speech is checked after orderly shutdown, not a live-file timeout. Real keyboard traversal also replaces an unreliable AT-SPI-only Wayland return-to-edit step. The experimental automatic polite announcement remains unqualified, not silently passed. Screenshots were visually reviewed; no human listening, physical GNOME/GPU certification, usability study or learning-outcome experiment was performed. Vite's ~695 kB minified chunk warning and unmeasured startup/RSS remain visible.

Artifact: `src-tauri/target/release/bundle/rpm/Memoquill-0.1.0-1.x86_64.rpm`, 3,438,200 bytes; SHA-256 `2964625472bf9455302b19039f15a4bcba8243ac7a066e5e7856c236fc41ec8f`. [Evidence snapshot](../quality/m1-desktop-evidence.json) fingerprints 42 source/config/test files. [Baseline, acceptance checklist and fallback](../quality/fedora-desktop-baseline.md); [reproduction](../../tests/qualification/README.md); [action log](../planning/m1-desktop-qualification.md).

Context7 was actually used on continuation to cross-check Tauri custom-command permissions, React commit/effect focus timing and ProseMirror's pre-parser DOM hook. Earlier official-source evidence was retained; there is no claim it had been used before the user's question.

Work stays in the original dirty `main` at `57d9e06e9e966622800f159ae0faa017a8c05ca1`. Pre-existing README/research/guidance changes were preserved; application/test files and documentation remain local/uncommitted. Scryntic repository and unrelated trackers were not modified. No commit, push, PR, merge, release or SonarQube run. Licensing work remains paused; root LICENSE hash is unchanged.

Next: review the concrete RPM/source and complete the baseline's manual Fedora/Orca checklist, then disposition KER-97. KER-98 requires a separate continuation instruction; this report does not start it.

Final documentation/source check: all checked local references valid; 42 source/config/test fingerprints and RPM hash match; LICENSE unchanged; Git whitespace check passes. Graph generation `2026-10-28T10:34:52Z` has no recorded code gaps on the evidence paths; non-tracked lock/config/asset metadata were verified directly. The unused 27 MB tool extraction and its never-started container were removed; candidate evidence/images remain, with no test container running.
