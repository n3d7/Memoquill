# Fedora desktop baseline — M1.1

Status: **automated candidate qualification passed; ready for review, not a release**. [KER-97](https://linear.app/kerigan/issue/KER-97/m11-qualify-the-fedora-desktop-and-accessibility-baseline) retains a manual desktop/Orca acceptance step before Done. M1.2 has not started. See the [action log](../planning/m1-desktop-qualification.md), [evidence snapshot](m1-desktop-evidence.json) and [reproduction instructions](../../tests/qualification/README.md).

## Implemented boundary

The pinned Tauri 2/Rust/React/TypeScript fixture provides prose editing, bounded MathML with readable LaTeX source, passive code, structural validation, native name-only file selection, reading preview, explicit operation results and system/light/dark appearance with 100–200% text. All application-authored UI/messages are English; materials and input tests support Unicode. Edits explicitly disappear when the window closes.

There is no persistent workspace, subject engine, learning history, scheduler, importer, AI runtime or MCP adapter. `QualificationDraft` is an ephemeral versioned boundary, not the future document schema. Rust rejects unknown fields/types/versions, NUL and oversized strings, then returns structural counts rather than correctness or mastery.

Only the main local webview can invoke the allowlisted draft check and explicitly requested picker. No raw filesystem, shell, HTTP server or arbitrary window-control permission is granted. Tauri custom commands are otherwise permissive: `AppManifest::commands` and negative mock-runtime tests address that distinction.

## Environment and artifact

| Item | Qualified candidate |
| --- | --- |
| Toolchain | Rust/Cargo 1.98.1; Node 22.23.1; npm 12.2.0 |
| Framework | Tauri Rust 2.12.5, build 2.7.4, CLI/API 2.12.4; React 19.3.1; TypeScript 7.0.2; Vite 8.3.5 |
| Runtime | Fedora 44 x86_64; glibc 2.43-9; GTK 3.24.52-2; WebKitGTK 2.54.2-1; AT-SPI 2.60.7; Orca 50.3 |
| Test desktop | Xvfb/Openbox 3.6.1; Weston 15.0.1/native GDK Wayland; IBus 1.5.34/libpinyin 1.16.5 |
| Fedora base digest | `de3396aa9d2452c5affd4a0b6eff9d3d0a7f3af58e243827790963c30e2d5e2c` |
| Installed image ID | `a4b923a6552615560e5964ccc5752d0f28b2214430d6fb32f3a83ad772c2a08c` |
| RPM | `src-tauri/target/release/bundle/rpm/Memoquill-0.1.0-1.x86_64.rpm`, 3,438,200 bytes |
| RPM SHA-256 | `2964625472bf9455302b19039f15a4bcba8243ac7a066e5e7856c236fc41ec8f` |

The host also has WebKitGTK 2.52.5; that is an observation, not the qualified runtime. The RPM was installed offline into a new image layer and launched in fresh private profiles. Updating Fedora packages/assets needs fresh evidence. This does not establish cross-machine binary reproducibility.

The rootless profile maps the user, drops capabilities, retains no-new-privileges, has no network/host desktop sockets, mounts only task input/output, and limits each run to 4 GiB/3 CPUs/100 seconds. The **outer container** uses `seccomp=unconfined` and `label=disable` so GTK's Glycin/bubblewrap can create its own namespaces/mounts. Host SELinux was verified **Enforcing**; no global policy/boolean/module was changed and component sandboxes were not explicitly disabled. This desktop-like profile does not qualify default confined containers, physical GNOME or hardware GPU rendering.

## Final verification

| Check | Observed result and limits |
| --- | --- |
| Build/type/unit | `npm run check` passes: Vite/TypeScript, 14 UI tests, Rust fmt/Clippy, 11 Rust tests, plus the targeted optimized glib regression. |
| UI accessibility | Labels/roles/feedback/focus and axe pass; jsdom contrast checking is disabled because it cannot measure pixels. Native initial/result/dark-200% screenshots were visually inspected. No WCAG certification claimed. |
| Native interaction | **35 assertions pass in each of X11 and Wayland**, including success/error results, editing after error, preview, picker cancel/select, focus return, themes/scaling and real IME. Protocol logs confirm native Wayland rather than Xwayland. |
| Orca | **7 presentation assertions pass per mode**: structural result, result context, validation error, requested cancellation feedback, formula source, passive code and selected filename/disclosure. Audio playback, pronunciation and comprehension were not evaluated. |
| Offline/network | Complete X11 flow traced through app/children with network disabled: no AF_INET/AF_INET6 socket creation or outbound address calls observed. Unix IPC/netlink discovery are expected. CSP limits connections to internal IPC; no remote frontend. Evidence covers the exercised paths. |
| IPC/ACL | Mock-runtime tests permit the main local webview and reject remote origins, another window and a registered-but-unlisted command. Packaged native success also passes; not every hostile IPC path is simulated. |
| Complexity/duplication | Application CRAP maximum **8**, all below 10 using real JS/Rust LCOV. Dryer has no candidates >=0.8. Native entrypoints are exercised separately from unit instrumentation. |
| Mutation | TypeScript **40 discovered sites killed**, zero survivors/uncovered sites. Corrected Rust worker **5 killed**; unchanged Rust evidence retained. Operators are limited, so these are not defect-freedom proofs. |
| Semgrep | 1.182.1, official TypeScript/Rust/Python rules, **26 authored files**, zero findings/errors. Tests included; CE inter-file coverage is limited. |
| Trivy | 0.75.0: no High/Critical or secrets. One Medium version-based glib advisory (mitigation below); three Low DS-0026 missing-healthcheck notices on one-shot images. No artificial healthchecks/broad suppression. Final source secret scan also has zero findings. |
| Dependency-Track | Existing 5.2.0 service completed dedicated `Memoquill / 0.1.0-m1.1` SBOM: 608 components, analysis after import, zero counters. Existing key cannot read detailed findings (403). Lockfiles still match that upload; no duplicate upload needed. Counters do not override Trivy's advisory. |

Vite reports a 695.32 kB minified JS chunk (212.71 kB gzip). Startup/RSS comparisons and large-collection budgets remain unmeasured. Static checks, native automation, human usability and learning-outcome evidence are distinct.

## Accessibility decisions and actual failures

1. ProseMirror's ordinary paste hook runs after HTML parsing. The earlier DOM event hook reads only plain text and prevents that parser path.
2. Native Tab traversal lost focus in preview. The shared read-only fixture dialog owns its one-control Tab loop, initial focus and close behavior. Reassess before adding interactive dialog content; this is not a general dialog system.
3. Picker restoration originally focused a still-disabled button. `usePickerFocus` waits for React's committed re-enabled control; no arbitrary application timeout was introduced.
4. Important feedback has an explicit path: Check opens an `Operation result` dialog; **Read last result** exposes picker/error feedback on demand. Closing returns to the exact initiating control. The polite status remains semantic; **automatic polite announcement is not qualified**.
5. Exploratory live-event capture saw children/defunct rather than text-insert events. The original live speech timeout was also unreliable because Orca buffers its debug file. Final audits run after orderly shutdown/flush. Do not cite that timeout as definitive upstream failure or as a passed automatic announcement. An experimental dynamic button description returned stale text and was removed.
6. AT-SPI focus alone could leave Orca's Wayland browse cursor elsewhere. The error-to-edit regression uses actual Shift+Tab traversal and verifies edited text. IME tests commit through private IBus, not synthetic DOM events. Workstation keyboard/language settings were not changed.
7. Initial generic Rust mutation overlays omitted bundled assets and could mistake build failure for a kill. The corrected private-worker wrapper supplies assets/entry files; exact Unicode-limit tests then killed a genuine boundary survivor. Only corrected results count.

The result pattern follows [W3C dialog guidance](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/). [WCAG 4.1.3 guidance](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html) still applies to future non-focusing feedback: dialogs everywhere are not a general substitute. Complex mathematical reading and multilingual speech remain separate evaluation work; visible source is retained.

## glib regression and mitigation

Registry `glib 0.18.5` crashed an optimized string-iterator regression with SIGSEGV under Rust 1.98.1. [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) and [upstream PR 1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343) identify the immutable C out-pointer error. GTK requires `glib ^0.18`, so a forced 0.20 replacement is not API-compatible.

Cargo pins [the reviewed backport](https://github.com/andrew-wommack-ministries/glib-rustsec-2024-0429-backport/tree/7e91f1a9420fd3e6c58b7913bbbf201888304ea9) at immutable revision `7e91f1a9420fd3e6c58b7913bbbf201888304ea9`. Earlier comparison of every tracked file with the published crate found only two mutable out-pointer changes plus a patch README. The optimized regression passes in the normal check command. ADR-08 records removal triggers/source-availability risk.

Version scanners still report 0.18.5: this exact advisory has source/regression mitigation, not a disappearance or broad waiver. RustSec also reports proc-macro-error as unmaintained. Licensing work/selection remain paused; prior artifacts are preserved and root LICENSE is unchanged.

## Gate disposition and manual review

Retain Tauri 2 for this **bounded candidate** with passing X11/Wayland/IME/Orca automation. Hold KER-97 at review rather than Done; do not start KER-98 in this assignment. Before closing manual acceptance, inspect the RPM/source and run on an intended Fedora GNOME session:

- Offline clean-profile launch; independently read/edit, check valid/invalid drafts, dismiss results and continue editing.
- Listen with Orca to results, **Read last result**, code and formula source; verify audio/usability rather than only presentation requests.
- Exercise preferred installed IME, picker cancel/select, Tab/Shift+Tab/Escape and interruption/resumption of the ephemeral window.
- Inspect light/dark/system, 100–200% text, focus visibility and the desired compositor/GPU. Do not treat fixture edits as saved learning data.

If a core workflow fails after one isolated reproduction and bounded app-versus-runtime diagnosis, stop expansion and compare the same fixture in Electron (pinned Chromium/AT-SPI) and GTK4 before revising ADR-02. Do not accumulate custom Orca patches, weaken global security or silently add a second stack. M7 still owns release-wide accessibility, hardware, recovery, performance and security qualification.
