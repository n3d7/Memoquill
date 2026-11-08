# X11-first platform sequencing

User decisions,2026-11-06: build the working application on Linux/X11 first;
defer Windows, macOS, Linux/Wayland, iPhone/iOS and Android; explicitly retain
future portability in the main stages. Current authorized work is docs/Linear
only. Application implementation, builds and GUI checks are paused.

Keep M1–M8 focused on X11, including the existing optional M8 scope. Put five
standalone tasks in a later inactive milestone after M8, with no parent or core
dependency links, dates or automatic execution. Do not turn port readiness into
early SDK setup, cross-builds, emulator tests or platform adapter implementations.

Retain the existing modular Rust/application core and typed frontend/controller/
native seams. Each main stage gains the concrete readiness criterion in the
[platform contract](../../architecture/platform-portability.md): portable identities,
formats and transactions; native I/O/permissions/lifecycle behind adapters;
input-independent UI commands and explicit optional capabilities. Existing Linux
I/O assumptions are recorded as debt, not represented as completed portability.

This approach follows the user's sequencing without creating platform forks or
a premature abstraction project. Parallel ports are excluded by the user; separate
domain/data implementations per OS would multiply migration/history/authority risk.
No framework replacement, plugin change, new runtime or product-code change is
part of this planning substep.

Preserve actual failures and unrun checks. Non-X11 qualification moves off the
critical path; X11 functionality, integrity/security, keyboard semantics and focus
remain required. Deferred KER-97 manual evidence stays unaccepted; full X11
assistive-technology/reflow checks remain release concerns in M7. No issue is
closed simply by changing scope. M2's revised X11 exit criteria need a separate
review after the user resumes implementation.

Update the roadmap, ADR-10, active quality/architecture scope and Linear main
milestone/issue criteria. Verification for this docs-only substep: source/coverage
evidence for existing seams, official Tauri adapter guidance, consistent scope,
valid local links and tracker readback of ordering/status/empty relationships.
Local completed-substep commit on dev only; no push/merge/release.
