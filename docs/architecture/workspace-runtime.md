# M1.2 workspace runtime

Implemented scope for KER-98; qualification/publication state is recorded in
[the execution ledger](../planning/m1-2-execution.md). This describes the current
storage foundation, not implemented subjects, imports, learning history or MCP.

## Boundaries

```text
React preferences -> four allowlisted async Tauri commands
                 -> typed ApplicationService -> bounded queue (64 requests)
                 -> one storage thread -> private Database/Owner/assets
```

`src-tauri/src/workspace` is one deep module with private storage submodules and
a public typed facade. Native adapters never receive a connection or generic
SQL/file executor. The existing ephemeral `check_draft` command remains separate.
The backend selects `app_local_data_dir()/workspace` with the unchanged app
identifier `io.github.n3d7.memoquill`; a caller cannot select another root.

The Rust service opens/audits the workspace off the GTK thread. Queue admission
is nonblocking (`Busy` when full); closed ownership has a distinct safe error.
An accepted write continues even if its reply receiver disappears. Shutdown
closes admission, drains accepted work, closes SQLite and releases the retained
OS lock before acknowledgement. The Tauri exit handler prevents normal exit
until that drain completes. SIGKILL/power loss cannot perform graceful cleanup.

Only the native adapter can construct a trusted UI `RequestContext`. It is not
deserializable. The public unverified-external context is denied, including
reads/reopen. This is a seam for [M6](ai-mcp-integration.md), not an implemented
grant system or an MCP server. No raw paths, principal strings or SQL occur in
the native request DTOs. Internal Rust asset registration accepts a bounded
stream from a trusted service; it is not an IPC command or importer.

## Storage and connection roles

Managed directories are private (0700); newly created lock/database/assets are
0600. The owner retains a no-follow, close-on-exec advisory file lock, never
unlinks it, and rejects public/symlink managed directories. Linux-specific file
operations are contained in `owner.rs`; private directory ancestors are trusted
operator configuration, not a sandbox against malicious code running as the
same OS user. Core content is neither encrypted at rest nor synchronized.

Schema 2 contains workspace identity, migration checksums, preferences,
operation receipts and asset metadata. These tables do not encode subjects or
learning proficiency. Stable UUIDv4 IDs and SHA-256 asset digests are distinct
checked types; wire contract version, schema version and package version are
independent. Revisions stay within exact JavaScript integer range.

Every connection receives FK enforcement, trusted-schema off, defensive SQL,
double-quoted string literals off and a 2-second busy timeout. Extension loading
is never enabled and its SQL entry point is tested as unauthorized. Existing
opens omit CREATE/URI and use NOFOLLOW/NO_MUTEX. The retained writer uses
WAL/FULL; the runtime must be at least patched SQLite 3.51.3. Current resolution
is rusqlite 0.41.0 / libsqlite3-sys 0.39.1, with actual SQLite **3.54.1** runtime,
source ID and compile options recorded by the engine test. JSON, FTS5 and real
FK rejection are probed; bindings/header constants are not runtime proof.

First creation writes a complete staged database in DELETE/EXTRA mode, applies
trusted embedded migrations and defaults, closes and fsyncs it, publishes with
Linux NOREPLACE, then syncs both changed directories. It never renames a live
WAL database. Existing files are preflighted for application ID, schema range,
workspace ID and embedded migration checksums before upgrade. Unknown/newer or
damaged workspaces are not reset.

Before an upgrade the SQLite backup API creates a bounded standalone snapshot
under `migration-backups/`, followed by integrity/FK checks and file/directory
sync. Migrations run in one immediate transaction with checksums/user_version.
These are internal database snapshots, not complete user backups of assets or
an export/restore UI; M3.2 owns that capability.

## Preferences and operation outcomes

Native commands: `workspace_status`, `save_reading_preferences`,
`operation_receipt`, `reopen_workspace`. Only the main local webview is allowed;
other windows, remote origins and unlisted commands are tested as denied.
Strict camelCase DTOs reject unknown nested fields, including forged principal
authority. Errors expose only a bounded code, never native paths/SQL/source.

Saving includes contract version, workspace ID, UUID operation ID, expected
revision and validated appearance/text scale. One immediate transaction checks
the exact prior receipt before current revision, then stores preference data,
new revision and receipt together. A same-ID/different-payload retry conflicts;
a stale revision does not overwrite newer data. Commit errors are conservatively
`OutcomeUnconfirmed`, not proof of rollback. The service closes uncertain state
until explicit reopen; it never blindly repeats an accepted request.

The UI loads saved preferences, applies local edits, and saves explicitly.
While unresolved, retry retains the entire original request/ID, reopens, then
replays it. Explicit reload fetches saved preferences and discards pending
appearance changes; material edits stay intact. Requested result dialogs make
feedback inspectable and restore the actual trigger's focus. Startup/storage
failure leaves the ephemeral material usable. No arbitrary error object is
displayed. The picker still discloses only a file name and reads no content.

## Asset publication and reconciliation

1. Stream into a unique private staging file with an explicit byte cap, hashing
   original bytes incrementally; never execute/parse them.
2. Fsync the complete file, then durably record a prepared operation.
3. Publish into `assets/sha256/<first-two-hex>/<remaining-hex>` without overwrite.
   Verify existing bytes before deduplication, including size/hash/no-follow.
4. Sync shard, asset-root and staging directories before a transaction commits
   the asset reference and applied receipt.

Startup interrupts prepared receipts without replaying their payloads. It
retains all source bytes/references and reports staged leftovers, orphan files,
missing/corrupt registered assets and unsafe paths. It does not delete, repair
or automatically garbage-collect them. Missing/corrupt/unsafe state blocks new
mutations; preferences and operation receipts remain readable. Unsafe shard
symlinks are inspected without following them. Explicit reopen rechecks state.

For this internal M1.2 slice startup performs a full streaming hash audit of
registered assets. **M3 must introduce and verify a large-collection audit
budget/UX before public imports.** This implementation makes no startup-latency
claim for large collections. Internal stream readers are trusted Rust callers;
future import/MCP adapters must add their own job/time/size/cancellation budgets.

## Verification and limits

Real temporary workspaces exercise ownership, restart, checksums/upgrades,
standalone snapshots, invalid FK writes, writer contention and SQLITE_FULL.
Injected boundary failures and killed child processes cover source publication,
prepared/applied receipts, lost replies, corruption and no-overwrite behavior.
Test-only failpoints/environment helpers are absent from production behavior.
The harness defaults to one test thread because forked test children can briefly
inherit other flock descriptions until exec/close; explicit contention/kill
tests still exercise real processes. See [flock(2)](https://man7.org/linux/man-pages/man2/flock.2.html).

These checks establish observed process-crash behavior and ordering, not physical
power-loss correctness, hardware honesty, all-filesystem guarantees, practical
learning effectiveness or human accessibility certification. Native RPM/AT-SPI
qualification and remaining gaps belong in the milestone evidence document.
