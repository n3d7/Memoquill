# Data model, durability, and ownership

This is a logical schema and contract plan, not applied SQL. Stable UUIDs identify entities independently of labels, placement, or source offsets. Use random UUIDs (no account/server required), timestamps in UTC, and explicit local display/study-day policy. One learner profile per local workspace is enough for V1; profile IDs keep personal history separate without implementing accounts.

## Core records

| Record group | Essential fields and relationships |
| --- | --- |
| Workspace/profile | ID, schema version, locale/timezone/study-day preferences; one learner identity, no login. |
| Subject/goals | ID, title, optional prior-knowledge note; goal IDs, criteria/outcome labels, archived state. No `subject_kind=english/python`. |
| Collections/entries | Arbitrary-depth collection parent within a subject; ordered entries reference resources/units. One containment parent per collection, multiple placements for shared content. Reject containment cycles. |
| Unit/path/relations | Unit ID, objective/goal references, resources/activities; ordered path steps; typed prerequisite/related/reference edges. Prerequisite DAG invariant, ordinary related links may cycle. |
| Resource/revision | Logical resource ID/current revision; immutable revision with authored/normalized document JSON, schema version, language, title, source revision and extraction references. |
| Asset/source | SHA-256, length, MIME, relative managed location; source origin/license/holder/attribution, original filename, source key, import timestamp and verification state. One byte asset may have several provenance records. |
| Anchors/annotations | Logical target and exact revision; page/block/line/offset plus quote/hash fallback, author and annotation text. Ambiguous reattachment is flagged, never silently guessed. |
| Activity/revision | Type ID, schema version, payload, outcome/evaluation policy and links. Question, answer, rubric, hints/solution belong to the immutable revision. |
| Session/items/drafts | Scope/configuration, instructional phase/practice or independent-check policy, frozen activity revisions/order, current item, reveal/hint state, recoverable draft with save sequence. Reading positions are separate preferences/state. |
| Attempt/evidence | UUID/idempotency key, profile, activity revision, session item, raw response, outcome/unknown, self-rating, feedback/rubric revision, assistance with observed/learner-declared/unknown provenance, duration/clock provenance and evidence method. Append-only; corrections are explicit superseding records. Unknown assistance never defaults to unaided. |
| Review state/events | Profile + activity + schedule generation; due instant, algorithm/parameter version, state; immutable review event and its originating attempt. Rebuildable state from events where supported. |
| Jobs/operations/proposals/grants | Job state/checkpoints/item outcomes, source manifests/transformation versions; operation ID/idempotency/principal and affected revisions; persistent draft overlay/revisions, proposal hash/bases/validation, specific approval or versioned delegation-policy decision, atomic budget consumption and application audit. Store grant token hashes separately from exported content. |
| Preferences/tags/search | Versioned preferences, tag membership, saved views; FTS and summary caches derived from canonical content/history. Grants/tokens are not learner-content exports. |

Use relational foreign keys and indexes for identities/ownership/order/relationships. Use JSON only for versioned document/activity/preferences payloads whose internal structure evolves, validated by application schemas. Do not turn the database into unvalidated generic objects. Cross-workspace references must be resolved by explicit import mapping, never dangling UUIDs.

## Revision and history rules

Content/reorganization operations never cascade-delete attempts. Each attempt preserves the exact prompt/solution/rubric it encountered. Archive hides material from ordinary selection; it retains references/history. A significant objective/activity change flags stale relevance and offers retain-with-warning, replace, or a new schedule generation. None erases historical evidence.

Human edits to derived lessons are separate from imported source revisions. Reimport matches a source key within an importer/source identity, compares hashes/revisions, and creates a proposal. Unchanged items retain IDs; ambiguous matches/new upstream IDs require user resolution. Annotations stay bound to old revisions until reattachment is verified. Multiple identical files can share bytes without conflating rights/attribution or logical resources.

Permanent deletion is an explicit workspace/content operation with a reference preview and backup implications. V1 can archive and remove unreferenced staged assets; physical removal of history-bearing data needs a deliberate purge workflow rather than automatic garbage collection. Do not claim secure erasure from SQLite, old backups, or storage hardware. Backup retention remains under user control.

## Transaction boundaries and file consistency

An attempt submission commits attempt + evidence + review event/state + session advance in one SQLite transaction; idempotency makes replay return the same result. A proposal/import application commits validated content/relation revisions + operation audit together. Conflicts or constraint failures roll back the entire selected change set.

SQLite and loose files cannot share one atomic transaction. Stage/validate assets under a controlled workspace directory, write/hash/fsync immutable assets, atomically rename them into their final content-addressed location on the same filesystem, and fsync affected parent directories (including newly created path metadata) before committing references. File fsync alone does not persist the directory entry; check all errors and test failpoints between each step. A crash may leave an unreferenced asset, but must never acknowledge a reference to an incomplete or non-durable path. Keep operation/journal records for recovery; garbage collection runs only after reconciliation and respects active backup/export leases. Disk-full/cross-filesystem rename failures are explicit failures. [Linux fsync semantics](https://man7.org/linux/man-pages/man2/fsync.2.html).

Enable foreign keys on each connection and verify them after migration/restore. Serialize writes; read snapshots and short transactions keep UI/search responsive. Foreign keys are not assumed enabled by default. [SQLite foreign-key documentation](https://sqlite.org/foreignkeys.html).

## Migrations and recovery

Ship ordered, checksummed embedded SQL migrations with tested fixtures from every supported prior schema. Back up before upgrading, reject unsupported/newer schemas, and migrate a safe working copy or within a transaction where supported. Do not run arbitrary migration text from an imported package. Failing migration leaves the prior usable workspace and a precise report; downgrade is restore-from-backup, not speculative reverse SQL.

Back up using SQLite's online backup API or a qualified equivalent, not a naked copy of the live main database while WAL is active. For a consistent **database + assets** backup, hold the application writer/asset-GC barrier while capturing the DB snapshot and its immutable asset manifest; acquire the lease protecting every referenced asset before releasing the barrier, then copy exactly those assets. Restore validates manifest, hashes, `integrity_check`, `foreign_key_check`, versions, and semantic references in a new directory before switching. [SQLite backup API](https://sqlite.org/backup.html).

Crash recovery checks unfinished operations, orphan staging files, DB integrity and missing assets; it reports/repairs only deterministic cases. A diagnostic or rebuild must not overwrite the only damaged copy. Search indexes can be rebuilt; personal attempts and originals cannot be reconstructed from a cache.

## Portable packages

Define `memoquill-package` format version 1: ZIP container, JSON manifest, UTF-8 JSON/NDJSON records, readable Markdown/CSV projections where meaningful, and originals/assets addressed by hash. Include IDs, relationships, revisions, schema/type versions, source attribution, counts and checksums. An optional history section contains attempts/events/schedules and explicitly selected preferences; content-only export excludes those by default and says so.

A full native backup is for exact restoration; a portable package is for interoperability and reimport, with loss/unsupported-type reports. Both must preserve all included canonical information.

| Data | Native full backup | Portable package |
| --- | --- | --- |
| Content/relationships/revisions, attribution and managed originals | Required | Required for the selected collection and its dependency closure; exclusions explicit. |
| Externally linked originals | Snapshot required for a complete backup; missing/omitted means incomplete | Selected snapshot or enumerated external-only/missing reference; no implied file durability. |
| Attempts/review events/schedule generations | Required | Optional explicit history section; preserve included state/versions and report unsupported data. |
| Preferences | Required | Explicit selected portable settings; machine paths may need remapping. |
| Drafts/session/reading resume state | Required | Only if the package schema/manifest includes it; default content export makes no promise. |
| Search/cache | Rebuildable, not recovery authority | Rebuild from included canonical data. |
| Agent tokens/grants | Excluded; reauthorization required | Excluded. |

Reject traversal, absolute paths, symlinks/hardlinks, duplicate/conflicting manifest paths, compression bombs, unknown executable handlers, unsupported versions, and checksum failures before activation. No token/grant secrets are exported; restored agents must be authorized again.

Stable IDs and revision provenance make future sync analysis possible, but V1 has no vector clocks, remote store, conflict-free algorithm, or sync service. A local operation audit is for recovery and explainability, not a claim that all future merge semantics have been solved.
