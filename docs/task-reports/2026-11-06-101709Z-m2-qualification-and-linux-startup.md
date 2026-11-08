# M2 qualification and Linux startup

Outcome: M2 implementation and review fixes are committed locally; full M2 acceptance remains incomplete.
Scope: [KER-99](https://linear.app/kerigan/issue/KER-99/m2-deliver-composable-lessons-subject-authoring-and-navigation), KER-116 and KER-118–120, dev; application source 5c72fdb. Local substep commits only, no push/merge/PR/release or remote CI.

- Backend/pins/drafts/receipts, frontend ports and the isolated constructor are implemented: eight components, four activity families, semantic edits/undo, preview/save/reopen and earlier immutable snapshots.
- Different lessons share no mutable drafts, content configuration, answers or help state. Linked-tab editing and its future seam remain excluded.
- Single integration review reused the reviewed backend baseline and found two Important issues. Fixes3b1c5a4/1c1fb43 anchor dirty forms and resolve published resource pins from validated durable drafts.
- New5c72fdb fixes the reproduced Linux/NVIDIA GBM white window by early process-local re-exec; explicit overrides are preserved. No host configuration, driver, dependency, licensing or CSP change.
- Passed:118 frontend tests/23 files with coverage;157 Rust library tests (3 ignored);4 Linux startup bin tests;TypeScript/build, final fmt/Clippy all targets; rebuilt RPM. Unchanged release glib regression reused (1 passed).
- Passed native X11 evidence on1c1fb43:60 main checks plus9 post-restart checks. Selection/native-role harness corrections and original failures remain recorded.
- Passed on actual host5c72fdb: ordinary launch without an exported renderer flag, confirmed by the user; saved lesson/draft/exercise UI, authored exact-answer preview, two hints, isolated tab state, save and published-answer semantics.
- User supplied four earlier screenshots; agents captured none. Control was returned when the user asked to use the PC; no new host restart/reflow/human speech evidence is claimed.
- Failed/unaccepted: nested Wayland WebKit self-SIGABRT; saved Orca audit missed five expectations. One cold-readiness probe timed out140 seconds before app launch, so its journey/audit were NOT RUN. Experimental readiness scripts are retained ignored, not shipped.
- Physical GNOME/human audible Orca, full visual/reflow acceptance and the failed isolated platform gates remain open. KER-99/KER-116 are not Done.
- Security: bounded Semgrep TS/JS scans and Trivy changed-file secret scans (including2new startup files) had0 findings; coverage/limits are in the [qualification ledger](../quality/m2-qualification.md). No full Rust SAST or vulnerability-free claim.
- Prior learning research was reused. Firecrawl Cloud read the [upstream Tauri issue](https://github.com/tauri-apps/tauri/issues/9304); SOFA supplied supporting environment-timing advice (Delphi API differs). Local reproduction established applicability, not source popularity.
- Immutable artifact/source digests and expected Tauri RPM bundle-marker patch are recorded under .qualification/m2-composition/linux-startup-candidate/. The newest RPM SHA256 is f9d77199107d508a9f71d7a136ee27b139720a7c57457e4033ce09d93531227f.
- Detailed test/review reports: .superpowers/sdd/2026-11-04-m2-composition/. Native logs/profiles: .qualification/m2-composition/. User preview remains separate from real user data.
- Pre-existing .agents task-workflow skill, .gitignore, AGENTS.md and two November1 tooling reports were preserved and are excluded from this commit.

Next: finish KER-97/M1.1 as explicitly requested by the user after this checkpoint, resolve the desktop/speech gates, then return to M2 acceptance. No M3 start.

Actual-use inventory for this qualification/fix phase: engineering-workflow; Superpowers TDD/review/verification; project workflow/context/reporting/tool-routing/learning-evidence guidance; SOFA technical/status/read and Firecrawl skills.
MCP/plugins: Serena, Codebase Memory, Linear, SOFA, Firecrawl Cloud. CLI/tools: RTK/Git, Cargo/rustfmt/Clippy, Vitest/TypeScript/Vite/Tauri, Semgrep/Trivy, rootless Podman, AT-SPI/IBus/Orca, strace/xdotool/i3, Python and RPM inspection.
Browser/screenshot tools and new arXiv/Consensus retrieval were not used in this phase; no SOFA writes, plugin updates, source uploads or system reconfiguration.
