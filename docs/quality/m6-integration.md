# M6 qualification

The candidate exposes published reads, owner-scoped durable lesson and course
drafts, checkpointed authoring, reviewed publication, bounded delegation and
compensating undo. The same course services own UI and MCP domain operations.
See the [implementation contract](../architecture/m6-local-mcp.md).

- Full verification passed 591 tests. It recorded zero failures and 28
  intentionally skipped process helpers. Build, formatting, Clippy, release
  regression, offline dependency scanning and license evidence passed.
  Doctest discovery reported no declared tests.
- The installed RPM passed 35 assertions in rootless Podman X11 with no
  network, default container seccomp, SELinux Enforcing and no screenshots.
  It covered explicit pairing, read scope, revocation, reviewed lesson and
  course groups, shared course material, delegated apply, idempotent retry,
  structural undo, closed-app behavior and socket cleanup.
- Both verification reports passed integrity audit. The native run used an
  isolated SSD graphroot after the default graphroot returned ENOSPC during
  an ID-mapped copy. The same local base image was used.
- Codex CLI 0.171.0 discovered both authoring tools through the trusted
  project-local configuration. The bridge accepted its observed
  `2025-06-18` initialization and returned `app_not_running` with the
  desktop closed. Separate Rust fixtures cover `2025-11-25`, modern
  `2026-07-28` and rejection of unsupported revisions.

Recover a schema 10 build from a verified schema 10 native backup.
Schema 11 adds authoring work, exact approvals and document provenance.
The delegated policy covers new material and changes within its own
submitted addition group. Changes to earlier or existing material require
trusted desktop review. Undo is available for unchanged structural additions
and declines when a later revision or protected learning reference exists.
