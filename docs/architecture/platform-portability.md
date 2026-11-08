# X11-first delivery and later platform adaptation

Decision: user instructions of2026-11-06. Current work is documentation/Linear
planning only; application implementation, builds and GUI qualification are paused.
When implementation resumes, the main M1–M8 cycle targets Linux/X11 only. Keep
M8's existing optional lesson-local scope; it is not a platform-porting stage.
Other platforms may be considered after that cycle and a separate instruction.

## Delivery boundary

- Current implementation/qualification target: the recorded Fedora/Linux X11
  baseline, initially RPM. Working offline authoring, study, ownership and recovery
  take priority. Other Linux distributions are not automatically qualified.
- No Windows/macOS/Wayland/iOS/Android adapter implementation, SDK installation,
  emulator/device setup, cross-build or target runtime matrix during M1–M8.
- Core stages must preserve portability now. This means stable contracts and
  separation of responsibilities, not speculative adapters or support promises.
- Keyboard access, semantic controls, focus recovery, readable content, X11
  regressions, security and durability remain core requirements. KER-97's deferred
  manual evidence is not a pass; full X11 assistive-technology/reflow acceptance
  remains a release concern in M7. Non-X11 qualification does not block core work.
- A failure exposing common data loss, unsafe authority or an X11 defect remains
  actionable in the main cycle; its discovery on another target does not exempt it.

## Shared core and replaceable boundaries

1. Keep subject/lesson/document/activity/history identities, validators, migrations,
   scheduling/rule logic and application transactions independent of display backend,
   window IDs, native widgets and OS paths. One versioned model, not a fork per OS.
2. UI consumes typed ports and controllers. Tauri IPC is an adapter. Per-lesson
   mutable forms, answers, hints and appearance remain isolated on every platform;
   shared service code never creates linked mutable lessons.
3. Native adapters own app-data roots, source selection/provider handles, controlled
   byte copying, safe opens/atomic publication/locking, permissions, clipboard,
   dialogs and lifecycle integration. Preserve these safety guarantees when changing
   APIs; a generic filesystem wrapper must not weaken them.
4. Durable schema/package contracts carry stable IDs, revisions, digests, explicit
   versions and declared capabilities, not absolute desktop paths or Linux error
   values. Future native errors map to the same bounded application error codes.
5. Persist drafts/receipts/session checkpoints independently of the lifetime of a
   window or process. Test interruption/recovery on X11 now; mobile suspension,
   process death and provider permissions need their own later device qualification.
6. Keep view models and semantic commands independent of mouse coordinates,
   hover-only actions or X11 shortcuts. X11 keyboard/UI behavior is implemented now;
   phone layout, touch, safe areas and virtual-keyboard adaptations remain deferred.
7. Discover optional platform capabilities explicitly. Desktop stdio, subprocesses
   and native file paths are not universal mobile capabilities. Future mobile work
   must adapt or clearly omit them, without mandatory cloud/accounts or an unscoped
   listening server. Domain authorization remains service-owned.

## Required readiness in the main stages

These are criteria for changes in each stage, not new platform implementations or
retroactive claims that an untested target works.

| Stage / primary Linear issues | Readiness required while implementing only X11 |
| --- | --- |
| M1 / KER-97,98 | Typed application/native boundary; platform-owned roots, locking, lifecycle and error translation. Preserve Linux safety while recording replaceable native assumptions. |
| M2 / KER-99 and children | Versioned compositions/documents/activities and migrations; replaceable UI/controller/IPC adapters; no display/window identity in lesson data; complete mutable lesson isolation. |
| M3 / KER-100,101 | Source/provider boundary with controlled immutable bytes; portable identifiers/manifests and safe relative package names; round-trip/recovery contracts independent of host paths. Portable package means a format contract, not qualified foreign-OS delivery. |
| M4 / KER-102,103 | UI-independent activity/session/evaluation/scheduling services; durable checkpoints and idempotent commands; clocks/lifecycle integration isolated from learning policy. |
| M5 / KER-104 | Semantic view models/commands, input-independent navigation and logical settings; layout/window metrics cannot become learning state. Mobile presentation stays a later adapter. |
| M6 / KER-105,106,107 | Shared scoped services separate from desktop stdio/jobs/native authority; explicit capability/version discovery, portable DTOs and stable errors; no assumption that mobile runs a desktop process/server. |
| M7 / KER-108 | Qualify/release only the supported Fedora X11 artifact; audit native assumptions and provide a port handoff with data/adapter/capability/test matrices. No foreign-OS build or support badge is required or implied. |
| M8 / KER-121 | Keep optional lesson-local rules/metrics versioned and UI-independent; implement only X11. No global mutable lesson links or platform-specific data fork. |

## Present evidence and known debt

At source b67f0fd, ContentPort/SourcePort/WorkspacePort and LessonPort are typed
frontend contracts; the Tauri builder registers native commands separately.
The lesson controller/renderer and saved-history boundary already provide reuse
seams. This is evidence of boundaries, not a cross-platform build qualification.

Native I/O still uses rustix, including Linux openat2 in attachments and native
safe-open/rename/owner/asset helpers. Those calls and their error conversion are
porting debt at the native I/O boundary. SQLite is bundled by Cargo, but that does
not qualify mobile linking, durability, lifecycle or sandbox behavior. Linux startup
bootstrap is target-gated in the binary; it must not become shared domain policy.
Do not introduce a second core/crate hierarchy or blanket filesystem abstraction
solely to create the appearance of readiness. Isolate new platform-sensitive work
at these seams as the relevant core stage is implemented; record remaining debt.

Tauri's [official mobile-plugin guide](https://v2.tauri.app/develop/plugins/develop-mobile/)
separates desktop/mobile implementations while allowing shared Rust logic. Its
[2.0 mobile-support account](https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/blog/tauri-2.0.mdx)
also warns that plugin availability differs. This supports retaining the current
core and adapters; it does not prove that Memoquill or every plugin supports all
targets, nor that ports will be quick or identical.

## Standalone deferred backlog

Linear milestone: **Later — platform ports after M8 (inactive)**, ordered after M8.
All five items are Backlog, low priority, unassigned, without cycles/dates/parents
or dependency/related links to the main steps. Descriptions prohibit automatic
execution during M1–M8. They do not gate core completion or publication.

| Item | Later adaptation/qualification |
| --- | --- |
| [KER-122](https://linear.app/kerigan/issue/KER-122/deferred-after-m8-windows-desktop-port) | Windows shell/runtime, safe I/O/locking/source pickers, offline package/recovery, keyboard/IME/focus/scaling and assistive technology. |
| [KER-123](https://linear.app/kerigan/issue/KER-123/deferred-after-m8-macos-desktop-port) | macOS shell/WebView, architecture/version choice, sandbox/I/O/lifecycle, offline recovery, VoiceOver and distribution decisions. |
| [KER-124](https://linear.app/kerigan/issue/KER-124/deferred-after-m8-linux-wayland-support-and-qualification) | Native Wayland/compositor/runtime/hardware, IME/portals/focus/scaling and assistive technology; carries the [saved crash diagnostic](../quality/wayland-deferred-diagnostic.md). |
| [KER-125](https://linear.app/kerigan/issue/KER-125/deferred-after-m8-iphone-ios-port) | iPhone/iOS touch/keyboard/layout, Files/sandbox/permissions, interruption-safe storage, VoiceOver, plugin/toolchain/device audit. |
| [KER-126](https://linear.app/kerigan/issue/KER-126/deferred-after-m8-android-port) | Android touch/back/keyboard, content-URI/scoped storage, process death/recovery, TalkBack and WebView/plugin/device audit. |

Future signing/accounts/store distribution, paid services, SDK installation and
device/system changes retain their normal authorization requirements. None is
authorized by creating these tasks. See [roadmap](../planning/roadmap.md), ADR-10
in [decisions](decisions.md), and the [Linear mapping](../planning/linear-backlog.md).
