# M1.2 — Durable workspace preparation

Historical preparation checkpoint. Later user instructions authorized implementation and, subsequently, PR/merge after qualification. Current status and authority are in [the execution ledger](m1-2-execution.md); the preparation-only instructions below describe that earlier phase.

Status: **preparation complete; implementation not started**. Prepared 2026-10-28 for [KER-98](https://linear.app/kerigan/issue/KER-98/m12-establish-durable-workspace-and-application-service-boundaries). The user's current instruction authorises investigation/planning and then stopping, not application changes, dependency installation, migrations, prototypes or another PR.

## Entry conditions and verified baseline

- Local `main` is clean at `1374c18decc30cb6081491707139b6910f8119fa`, the merged Foundation/M1.1 source from [PR #1](https://github.com/n3d7/Memoquill/pull/1).
- KER-98 is **Backlog**, blocked by KER-97, which remains **In Review**. M1.1 has passing bounded X11/Wayland/IME/Orca automation; physical Fedora/GNOME and human listening are still not confirmed. Before execution, explicitly resolve/disposition that gate and obtain the user's instruction to start. Preparation does not close either ticket.
- Current Rust is one `src-tauri` crate. `lib.rs` starts Tauri and exposes only `check_draft`; `draft.rs` is pure ephemeral validation. `src/bridge.ts` exposes that qualification contract. Cargo has no SQLite binding; no durable workspace/service layer exists. Do not persist `QualificationDraft` as the educational document model.
- Graph Tier 2: generation `2026-10-28T12:28:15Z`, eight relevant source/config paths and `src-tauri/src` scope have no recorded gaps/freshness mismatch. Source/config reads corroborate the landing points. A graph trace's unrelated Python `command` edge is not relied on as a real Rust caller; graph coverage is best-effort.
- Product/architecture sources remain [technical design](../architecture/technical-design.md), [data model](../architecture/data-model.md), ADR-04 in [decisions](../architecture/decisions.md), [MCP contracts](../architecture/mcp-contracts.md), and [verification](../quality/verification-strategy.md). This document refines execution, rather than creating another product architecture.

## What this step should deliver

| Include | Explicitly defer |
| --- | --- |
| One local workspace owner, private managed paths, typed IDs/errors and safe startup/shutdown | Accounts, profiles UI, cloud, sync, workspace-switching UI or a daemon |
| SQLite connection policy, initial schema, checksummed embedded migrations and internal pre-upgrade snapshots | Full user backup/export/restore packages (M3.2), imported SQL or arbitrary database access |
| Bounded storage request queue, operation receipts/idempotency and optimistic settings revision | Generic event sourcing, distributed transactions, CRDTs or an unrestricted command bus |
| Immutable content-addressed asset publication/reconciliation, tested internally | Format parsers, extraction/import UI, external linked-resource management, asset GC or media processing |
| Persist existing reading preferences through application services | Persisting lessons/attempts/reviews, subject tables (M2), scheduler/progress (M4/M5) |
| Narrow trusted UI adapter, tests reserving a denied external-principal path | MCP transport/server, grants/approval engine, agent tools or an in-app AI client (M6) |

The useful visible slice is persistent appearance/text size plus understandable startup/failure state. Material text remains explicitly ephemeral until M2. Keep the existing source picker name-only; do not silently turn it into an importer. UI/messages remain English, while arbitrary Unicode user content remains valid.

## Research findings that change implementation choices

| Finding and source | Decision / qualification |
| --- | --- |
| SQLite documents a WAL-reset race affecting unpatched releases through 3.51.2, with a fix in 3.51.3 and selected older backports. [WAL, section 11](https://sqlite.org/wal.html), [3.51.3 release](https://sqlite.org/releaselog/3_51_3.html). | Do not infer safety from the wrapper version or from one writer alone. Require a patched engine and record the resolved source/version and actual runtime before accepting the dependency. |
| The published rusqlite 0.41.0 manifest requires Rust 1.89 and libsqlite3-sys ^0.39.0; its README identifies bundled SQLite 3.54.0 for that pair. [Published source](https://docs.rs/crate/rusqlite/0.41.0/source/Cargo.toml.orig), [crate documentation](https://docs.rs/crate/rusqlite/0.41.0). | Candidate: exact rusqlite 0.41.0 with only needed features; pin the resolved dependency graph during authorised execution. This is not a completed compatibility/security audit or an installed dependency. |
| Pregenerated bindings describe a minimum API: the default libsqlite3-sys 0.39.0 `SQLITE_VERSION` constant reads 3.45.3, unlike the bundled engine documented above. [Constant](https://docs.rs/libsqlite3-sys/0.39.0/libsqlite3_sys/constant.SQLITE_VERSION.html). Context7 also returned an older 0.38.1/3.53.2 summary. | Published version-specific documentation takes precedence. Verify `sqlite_version()`, `sqlite_source_id()` and `PRAGMA compile_options` in the actual build; neither a binding constant nor an indexed summary identifies the running engine reliably. |
| WAL/NORMAL may lose recently committed transactions after system/power failure; WAL/FULL adds durable commit synchronisation. [SQLite synchronous](https://sqlite.org/pragma.html#pragma_synchronous). | Keep the existing WAL + FULL recommendation for acknowledged mutations. Do not weaken it for an unmeasured performance benefit. Hardware/filesystem honesty remains an assumption to qualify. |
| rusqlite `Connection` is Send, not Sync; NO_MUTEX is the normal Rust ownership model. Default open flags also include CREATE and URI; NOFOLLOW is available. [OpenFlags](https://docs.rs/rusqlite/0.41.0/rusqlite/struct.OpenFlags.html). | One connection-owning thread; explicit open roles/flags. Existing workspace opens must not create a database or accept URI parameters. No shared-cache or FULL_MUTEX workaround. |
| Rust file locks are available in the existing toolchain; Linux uses advisory flock-style semantics and locks are tied to handles. [File locking](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock). | Prefer `File::try_lock` and a retained owner handle over PID-file existence or another locking crate. Never unlink a held lock file; PID text is diagnostic, not authority. |
| File fsync does not itself persist the directory entry; directory fsync and error handling are required. [Linux fsync](https://man7.org/linux/man-pages/man2/fsync.2.html). Linux no-replace publication is exposed by rustix. [renameat_with](https://docs.rs/rustix/1.1.6/rustix/fs/fn.renameat_with.html), [NOREPLACE](https://docs.rs/rustix/1.1.6/rustix/fs/struct.RenameFlags.html#associatedconstant.NOREPLACE). | Publish only complete, synced managed files; sync changed directories before committing references. Do not use ordinary replacing rename for immutable assets. Keep Linux mechanics in a small adapter. |
| SQLite's backup API produces a consistent database snapshot; copying only the live main file is unsafe as a general WAL backup strategy. [Backup API](https://sqlite.org/backup.html). | Use the API for an internal pre-migration database snapshot. This is not a complete user backup of assets/history or a substitute for M3.2. |
| A peer-reviewed crash-consistency study examined six Linux filesystems and eleven systems, finding protocol sensitivity and 60 vulnerabilities. [Pillai et al., OSDI 2014](https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai). | Supports explicit ordering/fault tests, not a claim about today's Fedora filesystem correctness. The inspected abstract is historical evidence; no reproduction or new experiment was performed. |

These are documented engineering facts and recommendations, not new learning-science findings. No Consensus quota was spent. The focused retrieval used Context7 and public official sources through the existing self-hosted Firecrawl. Its scraper rejected the advertised `toolDetail` argument; removing that optional argument resolved retrieval without reconfiguring a service.

## Recommended structure and contracts

Keep one Rust crate and a modular monolith. Proposed responsibilities, not directories created in this preparation:

```text
src-tauri/src/
  lib.rs                  thin Tauri setup/commands
  domain/                 typed IDs, reading preferences, revisions, safe errors
  application/            facade, trusted context, typed requests and receipts
  storage/                workspace lifecycle, SQLite, migrations, assets, recovery
  platform/               private paths, retained file lock, no-replace publication
```

The dependency direction is adapter → application → storage/domain. Do not export a generic SQL executor, arbitrary file reader or closure-based database command bus to adapters. Concrete private modules are sufficient; add a trait only for a real boundary such as the few publication I/O calls requiring error injection.

Use stable UUIDv4 identities with distinct Rust types (`WorkspaceId`, `OperationId`), lowercase canonical wire strings and positive integer revisions. UUID ordering is not creation time. Use SHA-256 asset digests as a different type, never interchangeable UUIDs/paths. Retain request/type versions independently of database schema and app package versions.

The initial schema needs workspace identity/format metadata, migration checksums, versioned preferences, asset metadata and operation receipts. It does not need subjects, questions, users/accounts or history tables yet. Keep workspace identity authoritative in SQLite, rather than duplicating it in a second mutable manifest.

`RequestContext` is constructed by the adapter: trusted UI identity, workspace identity and operation ID. Do not accept a client-provided principal, grant, permission flag or filesystem root. Reserve an external-principal verification seam but reject unverified external access; M1.2 is not the implementation of M6 authority policy.

Suggested UI commands are workspace status, read/save reading preferences and lookup of the caller's operation receipt; retain the existing qualification check separately. New commands must be explicitly added to AppManifest/capabilities. Keep Rust/TypeScript DTOs small with strict unknown-field rejection and checked wire fixtures; no schema/code-generation framework is necessary yet.

Preferences version 1 covers existing theme (`system`, `light`, `dark`) and text scale (100/125/150/200), with a row revision. A save includes its expected revision; unrelated future content edits must not invalidate every preference save. A successful reply means durable commit. Errors use stable codes such as already-open, busy, invalid-request, revision-conflict, unsupported-schema, recovery-required and outcome-unconfirmed, with safe English UI mapping and no raw SQL, tokens or payload dump.

## Workspace and storage policy

Resolve the root through Tauri's app-local data path for the existing identifier `io.github.n3d7.memoquill`, then use a single `workspace/` subdirectory. Preserve native `PathBuf` values; do not hardcode `/home/void` or lossy UTF-8 paths. Keep caches/logs outside canonical data. [Tauri paths](https://docs.rs/tauri/2.12.5/tauri/path/struct.PathResolver.html). In tests inject a fresh private temporary root; never use a developer's normal profile.

Proposed layout: `workspace.lock`, `memoquill.sqlite3`, SQLite-owned WAL/SHM sidecars, `assets/sha256/<first-two-hex>/<remaining-hex>`, `staging/`, and `migration-backups/`. Managed directories are private (0700), newly created data files 0600. No live WAL/SHM file is manually deleted. Local filesystems are the supported starting point; network/shared-drive workspaces are not qualified.

Acquire the nonblocking workspace lock before database access and retain its file handle for the owner's lifetime. A second cooperative process reports already-open and does not touch the database. A focus-existing-window plugin is optional UX, not storage authority; no permanent daemon, new HTTP endpoint or PID-file lock scheme is needed. This is not a defence against a fully compromised OS account bypassing application ownership.

One dedicated storage thread owns the writer connection and private store. Use the already-present Tokio 1.54.1 bounded mpsc/oneshot primitives with async callers and synchronous receiving on that thread, without another runtime. [Tokio channel documentation](https://docs.rs/tokio/1.54.1/tokio/sync/mpsc/index.html). Initial queue capacity 64 and 2,000 ms SQLite busy timeout are **engineering starting values**, not validated performance targets. A full queue rejects safely instead of accumulating unbounded work. Never perform blocking SQLite/fsync on the Tauri/UI thread merely because a command is declared async.

M1.2 internal fixture assets may use that storage thread; copying/hashing must happen outside a SQL transaction. Preserve an opaque prepared-asset seam so M3 can move expensive preparation to its bounded workers before exposing bulk ingestion. Do not enqueue raw large files or build a job framework in this step.

Connection roles are explicit: new-bootstrap, read-only preflight/snapshot, runtime writer and backup destination. For trusted runtime writes verify foreign_keys=ON, WAL, synchronous=FULL and the selected busy policy. Do not rely on defaults. Read-only/newer-schema checks must not initialise/migrate a database or switch its journal mode. NOFOLLOW protects the final database filename; it is not a proof against every same-account path race. Disable URI input/shared cache and extension-loading access; consider defensive/trusted-schema settings with actual migration/capability tests, not assumptions.

## Initialisation, migrations and recovery

New workspace initialisation must not turn a crash-created empty file into an invitation to overwrite an unknown database. Recommended approach: build the initial database privately in staging, with the embedded initial migration, application marker and workspace ID; use rollback DELETE/EXTRA for this temporary database, close it, sync its complete file, then publish without replacement and sync the affected directories. Only then open the published database with runtime WAL/FULL. This special case is a **closed, standalone new database**, never renaming a live WAL database.

For existing workspaces, validate application marker and supported format/migration checksums before mutation. Refuse unknown/newer schemas and preserve the original. The exact SQLite application marker is chosen once during implementation and is not a replacement for a schema version.

Ship ordered embedded, checksummed SQL migrations. No imported SQL, URI script, filesystem SQL file or automatic down-migration is allowed. Under the owner barrier, create a verified standalone pre-upgrade snapshot with the backup API; close/sync that destination and its parent. Then run the trusted schema change and version/checksum bookkeeping atomically. Do not start a write transaction on the same source connection before a backup API call that cannot support it.

On migration failure, roll back and retain the prior workspace plus snapshot. Test real foreign-key and integrity checks, not only PRAGMA values. ENOSPC may require freeing space before reopening; do not promise a usable disk while it remains full. If outcome/rollback cannot be established, preserve files and enter recovery-required instead of silently resetting.

Startup reconciliation classifies incomplete operations, unreferenced published assets, staging leftovers and missing/corrupt registered assets. It never executes journal payloads as SQL/code. Recover deterministic states; report ambiguous/corrupt states without overwriting the only copy. Automatic asset GC, backup leases and full restore UI are deferred, so orphan retention is preferable to premature deletion.

## Asset publication and operation receipts

The service receives a controlled byte stream or opaque staged token, never a client-selected destination path. Every staging call requires a finite byte limit; format/user-facing limits belong to the later importer. Paths are derived only from validated IDs/digests.

1. Validate context, typed request and limits. Check the operation key under workspace/principal scope. The same key with another canonical request hash is a conflict.
2. Record preparation when needed; this is not successful application. Stream to a newly created private staging file, hash/count it, flush buffered writes and sync the file. Check every error.
3. Create the controlled shard if needed and persist its parent metadata. Publish the complete file on the same filesystem with no-replace semantics (candidate rustix 1.1.6 `fs` API). If a destination exists, verify its bytes/hash/size; a mismatched file is corruption, not a successful deduplication or permission to replace it.
4. Sync the final directory and every newly created/changed parent; sync staging directory removal as relevant. No database reference may commit before these durability steps complete.
5. In the final SQLite transaction, recheck revisions/context as applicable, register references/application changes and store the terminal operation receipt together. Only after successful durable commit acknowledge success.

| Interruption | Permitted recovery outcome |
| --- | --- |
| During staging or before publication | No committed reference; retain/classify incomplete staging |
| After publication but before SQL commit | Complete unreferenced asset may remain; no fabricated success |
| During SQL transaction | Rollback or a committed state determined on reopen; no half-applied logical update |
| After commit but before response | Same request key returns its stored receipt without applying again |
| Uncertain commit/failed acknowledgement | Keep the operation ID; query/reconcile after reopen, never retry under a fresh ID blindly |

For new operations, revision checks happen before mutation. For an already-applied exact retry, return the receipt after validating caller/context rather than fail because the old expected revision has already advanced. A prepared/interrupted record is not a resumable arbitrary command: return a deterministic interrupted/recovery result; later job orchestration can use its own child operations. This receipt journal is not an event-sourcing reconstruction engine. A dropped client does not imply cancellation of a mutation already accepted by the owner.

## Execution order once separately authorised

| Increment | Concrete result / exit evidence |
| --- | --- |
| G0 — entry check | Re-read source/issue/user authority, disposition the M1.1 manual gate, preserve current worktree; select/pin the dependency pair only then |
| G1 — owner and contracts | Typed IDs/errors/context and private root/retained lock; two-process ownership and lock release after crash; no UI/DB backdoor |
| G2 — database lifecycle | Connection factory, runtime engine/options evidence, safe first creation, initial migration/checksums, newer-schema refusal and pre-upgrade snapshot/rollback tests |
| G3 — first useful durable slice | Versioned preferences through the facade and operation receipts; restart, stale-revision, exact-retry and conflicting-key tests |
| G4 — assets and recovery | Internal bounded staging/publication/reconciliation, then failpoints around file sync, publication, directory sync and SQL commit; no format importer |
| G5 — thin UI integration | Async status/preferences adapters, English startup/busy/recovery messages, safe retry and focus; disclose material edits still ephemeral; keep picker name-only |
| G6 — candidate qualification | Focused contracts/fault/property tests, native regression/RPM/private XDG checks, required security/quality tools and honest remaining limits; review without auto-starting M2 |

Begin G1 with a failing ownership/isolated-workspace test and the smallest vertical result; do not implement all tables/UI at once. G3's public UI change follows the lower-level reliability tests. Keep changes reviewable without making each row a separate Linear ticket or inventing time estimates.

## Verification and tools

| Risk | Required useful test |
| --- | --- |
| Two owners, path mistakes | Separate processes contend for one root; a killed holder releases the OS lock; no changes outside injected test roots; symlink/URI/unknown-workspace rejection |
| SQLite configuration/integrity | Invalid FK writes actually fail; new connection policy cannot drift; runtime SQLite version/source/compile flags and JSON/FTS capability probes are recorded |
| Migration damage | Fixtures for every supported version, altered checksum/newer version, deliberate SQL failure, busy writer and real SQLITE_FULL (e.g. bounded max_page_count); old logical data survives |
| Asset/reference divergence | Real temporary-file round trips plus injected write/sync/rename/directory failures; same-digest duplicate verification; never a successful reference to incomplete bytes |
| Lost reply/double write | Kill a test child before/after commit and reopen; same ID replays one receipt and differing payload conflicts; no false success during an uncertain result |
| Unbounded work/UI freeze | Bounded queue/closed-owner behavior; no raw payload buffering; async UI pending/failure states preserve edits and focus |
| Authority bypass | Other window/remote-origin/unlisted commands denied; payload cannot forge a principal or destination; future external access remains denied |
| Schema subject bias | Preference/asset/operation contracts accept Unicode and have no English/Python/card-specific entity branches |

Use test-only child-process helpers and private I/O failpoints; production binaries must not contain an environment-triggered crash, arbitrary path or SQL escape hatch. Property tests are appropriate for canonical IDs, limits and operation/state invariants; add proptest only if its generated cases reduce real risk. Subprocess kill tests establish process-crash recovery, not physical power-loss correctness. Qualify the observed Fedora local filesystem; stronger kernel/power-failure and multi-filesystem claims remain M7 work.

The workflow owner during implementation remains the user's **ub-mode**. Existing project workflow/reporting/routing and skill-catalog support it; do not add a second planning/TDD/review loop. Context7 plus focused official Firecrawl reads supply API facts. Codebase Memory/coverage and source reads supply code evidence; Serena is already activated. No new MCP server or plugin is needed.

Existing CLI tools are present: Crapper, Dryer, Mutator, Semgrep, Trivy and gh. Their last qualification results are not new results for future M1.2 code. On execution run proportionate Rust/TS/build/ACL/integration/fault checks, real LCOV → CRAP <10, Dryer >=0.8 review, valid mutation overlays, local Semgrep and Trivy, then an updated SBOM/Dependency-Track analysis if dependencies changed. The prior Dependency-Track service/limited findings permission is known from M1.1, not revalidated now. SonarQube remains excluded; do not rerun native/scanner work during this preparation or claim it already validates a nonexistent implementation.

No unresolved product choice requires user input to finish this preparation. The significant pending input is permission to implement and the M1.1 gate disposition. Licensing work/selection remains paused; no manifests, lockfiles, license payloads, database files or application/test source changed in this preparation. Stop here.
