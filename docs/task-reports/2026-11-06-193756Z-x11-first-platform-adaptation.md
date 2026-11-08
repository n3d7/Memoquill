# X11-first core and deferred platform adaptation

Completed the requested docs/Linear planning substep. The latest user instruction
pauses application development, builds and GUI qualification. No application code
changed in this substep; source remains b67f0fd. Main M1–M8 targets X11 only, with
concrete portability acceptance inside the main steps rather than early ports.

- Added [ADR-10 and the platform contract](../architecture/platform-portability.md)
  and a [sequencing design](../superpowers/specs/2026-11-06-x11-first-platforms-design.md).
  Retain one domain/data model, typed UI/controller/service contracts and native
  I/O/permissions/lifecycle/transport adapters. Document actual rustix/Linux debt;
  Tauri/mobile support is not a promise that this app or every plugin is portable.
- Updated all8 main Linear milestones,13 primary issues and5 M2 integration/review
  issues with X11 scope, stage-specific readiness and the current application pause.
  Existing completion states are unchanged. KER-97 remains In Review; KER-99/116
  remain In Progress; revised X11 exit criteria need review only after user resume.
- Created KER-122 Windows,123 macOS,124 Wayland,125 iPhone/iOS and126 Android in
  **Later — platform ports after M8 (inactive)**. Readback verifies ordering after
  M8 and Backlog status, no parent/assignee/cycle/date/dependency/Related links.
  Linear created unwanted Related links from issue-number mentions; removed both
  the mentions and links and verified the standalone result. No automatic start.
- Amended roadmap/product/technical/quality scope, M2 contract/plan, M1 deferral,
  backlog mapping and handoff. Preserve X11 integrity/security/keyboard/semantic
  focus and M7 release-level AT/reflow requirements; do not pass deferred evidence.
- Retained the [Wayland diagnostic](../quality/wayland-deferred-diagnostic.md)
  completed before this hold: a direct attribute query reproduces the WebKit
  overflow in the app; ordinary minimal controls do not. Related upstream PR is
  not a verified installed-runtime fix. Diagnosis is deferred; CSS experiment
  was never applied. No host desktop/package/configuration change.

Verification: source snippets plus coverage for the relied-on paths; official
Tauri mobile-adapter guidance through Context7/Firecrawl Cloud; full issue/milestone
readback; local docs/reference/scope checks and Git diff checks. Application tests,
builds, security scans and GUI checks were not rerun for this docs-only change.
Earlier native/automated evidence is reused without new learning or platform claims.

Owned changes are the architecture/scope/planning/quality documents above. Preserve
pre-existing AGENTS.md, .gitignore, the project-workflow skill and two November1
tooling reports. Base b67f0fd on dev; one local completed-substep commit only;
no push/merge/PR/release. Next: none for this planning request; await explicit app-work resume.

Used: Superpowers brainstorming/design and verification; project workflow/context/
reporting; Codebase Memory, Linear, Context7 and Firecrawl Cloud; Git/RTK/Python.
The retained preceding diagnosis used private rootless Podman, AT-SPI and strace.
No private source upload, new dependency, SDK/platform build or scientific recollection.
