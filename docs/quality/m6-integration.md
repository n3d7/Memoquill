# M6 qualification

## PDF fallback in Journey C

Generate a fresh synthetic profile from the trusted workspace service before
running the installed desktop journey. The seed test verifies the stored
original and exports only when explicitly requested. Use a fresh
checkout if `.qualification/m6-seed` already exists. These commands assume
the local verification runner is available under `tools/verification`.

```sh
MEMOQUILL_EXPORT_M6_FIXTURE=1 python3 tools/verification/verify.py run \
  --profile affected --rust m6_pdf_journey_seed
mkdir -p .qualification/m6-native-input
cp tests/fixtures/m3/original.pdf .qualification/m6-native-input/
tar -C .qualification/m6-seed -czf .qualification/m6-native-input/m6-source-profile.tar.gz \
  io.github.n3d7.memoquill
python3 tools/verification/verify.py run --profile native --scenario mcp \
  --native-image localhost/memoquill-m1-desktop:20261027 \
  --native-input .qualification/m6-native-input
```

The installed journey publishes a typed citation with desktop review, checks
the attachment-only MCP result, compares the disclosed original with the
fixture digest and bytes, and checks denial for another lesson and after
revocation. It continues into offline study. The prepared profile isolates
these checks from the container's GTK file chooser and its icon loader.
Run the scenario with default container security, SELinux Enforcing,
networking disabled, and screenshots disabled.

## Merge candidate follow-up

- Focused service fixtures cover four subjects with goals, prerequisite
  links, shared materials, and recall, choice, constructed, and procedure
  activities. They verify inert draft previews and denial of delegated
  revisions to shared references.
- Delegation fixtures cover the separate document permission, expiry,
  owner mismatch, manual edits, stale undo, and faithful compensation of
  newly created documents without removing revision history.
- The installed RPM passed 44 assertions in isolated Podman X11, with
  SELinux Enforcing and screenshots disabled. The journey includes exact
  checkpoint recovery after reapproval, a later delegated document edit,
  compensation, and ordinary offline practice after revocation.
- The synthetic PDF fixture verifies attachment-only extraction, preserved
  original bytes and digest, and lesson scope through the application
  services. The native file chooser could not open under this container's
  GTK icon loader, so PDF evidence comes from the service fixture.

## Earlier candidate (`56ac41f`)

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
