# Native selected-course recovery package v1

G1 native recovery is implemented at the Rust service boundary. G2 portable
content packages, native file authority/pickers and renderer review flows are
separate pending work; this backend does not complete M3.

`ApplicationService::capture_native(context, source_course_id)` queues capture on
the existing serialized writer. SQLite's backup API captures committed WAL data;
every registered immutable asset is copied and verified before that barrier is
released. The opaque `NativeCapture` owns a private 0700 temporary directory.
`write_to<W: Write + Seek>` builds the ZIP afterward. The caller owns destination
authorization, cancellation and atomic output publication; a failed write can
leave its supplied writer partially written. No renderer path is accepted.

The ZIP contains `manifest.json`, `database.sqlite3`, and `assets/<sha256>`.
The camelCase JSON manifest declares formatVersion1, kind `nativeFull`, scope
`selectedCourse`, exact schemaVersion7, databaseSectionVersion1 and
assetsSectionVersion1, sourceCourseId, workspaceId, complete, missingAssets,
omissions, tableCounts and payloads. Each payload has name, sha256 and byteLength.
The database includes every current table and its IDs, revisions, original-source
provenance, lesson/source bindings and pinned revisions, drafts, ingestion jobs,
operation receipts, preferences, annotations and reading positions. No unrelated
course, catalogue, credentials, migration backup, lock or runtime grant is copied.

All registered originals are included when available. A missing/corrupt/unsafe
original is listed by its expected digest in missingAssets and complete is false.
Database references remain intact. A package's hashes establish consistency,
not authorship or authenticity. Packages are passive local data, never executed
or uploaded by this implementation.

`NativeCapture::read_from<R: Read + Seek>` bounds actual ZIP bytes before parsing.
V1 rejects ZIP64, multidisk, prepended/SFX data, invalid/undeclared names,
duplicate names, overlapping entries, symlinks/special types, encryption and
methods other than stored/deflate. EOCD plus bounded central/local header checks
run before ZIP metadata allocation. Archive paths are compared with a closed
grammar; only generated destinations under private staging are written. There
is no generic ZIP extraction.

| Bound | V1 limit |
| --- | --- |
| Actual archive | 512 MiB |
| Expanded total | 1 GiB |
| SQLite snapshot | 256 MiB |
| Individual original | 64 MiB |
| Entries | 4098 |
| Manifest / central directory | 2 MiB each |
| Entry name / extra / comment | 128 / 1024 / 1024 bytes |
| SQLite schema objects | 128 |
| Rows per table / all tables | 1,000,000 / 2,000,000 |
| SQLite text/blob cell | 8 MiB |

The staged SQLite connection is read-only with trusted_schema disabled,
defensive controls and extension loading disabled. Before domain reads, compare
all sqlite_schema objects, including indexes, triggers, views and FTS objects,
with a separate database built only from trusted bundled migrations. Require
application ID, exact schema7, migration checksums, integrity/FK checks,
workspace identity, manifest table counts, bounded records, existing organization,
document and lesson validators, and domain reference consistency. Future required
sections/schema versions are rejected. Source schema SQL is never executed as
a migration or trusted merely because its hash was recomputed.

`CourseManager::restore_native(operation_id, title, &package, allow_incomplete)`
admits a new course only. Default callers must pass false for incomplete recovery;
true represents an explicit review confirmation and the result remains incomplete.
Internal workspace/content/history IDs are preserved. The operation UUID is the
new catalogue course ID; `NativeRestoreReport` returns sourceCourseId→courseId,
workspaceId, complete, missingAssets and omissions. Existing courses are untouched.
The existing 32-course cap, exclusive owner and shutdown checks remain.

A private durable `courses/restore-operations/<operation UUID>.json` intent binds
operation ID, canonical manifest SHA256 and title before root publication. The
staged root contains the identical `restore-intent.json` marker. Files and all
affected directories are synced, then the root is published without replacement.
Catalogue insertion marks it initialized and selects it in one transaction. A
retry with the same identity/title either returns the admitted result or verifies
the owned pending root before admission. Changed payload/title conflicts; unknown
roots are preserved and rejected, including byte-identical roots without the
marker. Crash leftovers are not silently adopted or deleted. Normal create/service
orphan-root guards are unchanged. Restore journals are private runtime evidence,
not included inside exported packages.
