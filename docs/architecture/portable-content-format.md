# Portable content package V1 (G2)

G2 is a selected-course content exchange ZIP, kind `portableContent`, scope
`selectedCourse`. It is not native recovery. It carries no SQLite database.
The existing G1 format remains separately available for native recovery.

## Public native interfaces

- `ApplicationService::capture_portable(RequestContext, source_course_id: Uuid,
  include_originals: bool) -> Result<PortableCapture>` is async, serialized through
  the selected course writer. It copies originals before releasing capture authority.
- `PortableCapture::manifest() -> &PortableManifest`, `write_to<W: Write + Seek>`
  and `read_from<R: Read + Seek>` expose no filesystem paths. Read returns only a
  fully verified opaque package; archive publication remains the native adapter's job.
- `CourseManager::import_portable(operation_id: Uuid, title: String,
  &PortableCapture) -> Result<PortableImportReport>` always creates a new course.
  The operation UUID becomes the new catalogue ID. Internal workspace, content,
  revision and pin IDs remain stable within the isolated course. No existing course
  is overwritten or merged. A durable intent binds the operation to the exact
  typed manifest digest and title, including across restart and publication failure.

## ZIP members and bounds

`manifest.json` is strict camelCase JSON, formatVersion 1. Its required `sections`
map is exactly `{content: 1, readable: 1, assets: 1}`. `requiredAdapters` is empty.
Any required future history/attempt/schedule adapter or unknown section/version is
rejected. This reserves a future M4 contract; it supplies no M4 runtime or data.

`content.ndjson` is required. Each UTF-8 line is a strict JSON object
`{"family":"resources","record":{"id":"…","title":"…","archived":false}}`.
The final newline is mandatory for nonempty content. Blank, truncated, unknown or
malformed records fail. Family names and typed field schemas are the fixed
compile-time declarations in `packages/portable/records.rs`, never package-supplied
SQL, table names or DDL. JSON domain values include typed Document, Composition
and SourceBindingRevision; activity definitions use the existing domain validator.

`readable/content.txt` is a generated-name plain-text projection with resource
names, all immutable document revisions, lesson compositions and source attribution.
Structured documents/compositions are pretty JSON inside the text, retaining all
unsupported content without executing markup. It is derived, never authoritative;
reader checks exact canonical projection digest/length. Optional `assets/<sha256>`
contains original bytes only when explicitly selected at export.

Actual ZIP <=512 MiB; expanded <=1 GiB; <=4098 members; manifest/central directory
<=2 MiB; names <=128 bytes. Content and readable text each <=64 MiB; each NDJSON
record <=2 MiB, <=20000 records per family and <=100000 total. Assets <=64 MiB each.
ZIP64, multidisk, prepended data, encrypted/special/symlink entries, duplicate names,
overlap, unknown members, traversal and hash/size mismatches fail. Metadata is
bounded before ZIP directory allocation. Every declared member has SHA256 and size.

## Included and omitted families

The all-course shareable subset includes subjects (without personal prior knowledge),
goals, collections, resources, units, unit-goal/resource links, placements, paths,
path steps, relations, tags and tag-resource links; every resource revision/head;
source revision counters and original source metadata; every published lesson
revision/head, activity revision/head, resource owner, dependency and source binding.
Referenced asset metadata follows content/source closure; unrelated assets do not.
Historical and non-head authored revisions are included, including old pinned ones.
`includedRecords` enumerates every family count. `authoredVersions` separately counts
resourceRevisions, lessonRevisions, activityRevisions, sourceBindings and
originalSourceVersions. An unsupported row fails export/import rather than vanishing.

`omissions` explicitly declares personal annotations, reading positions, learner
responses/progress, reading preferences, private lesson drafts, ingestion jobs,
operation receipts, secrets/grants/tokens, unrelated courses/catalogue, subject prior
knowledge, unreferenced assets, derived search indexes and unimplemented M4
attempts/history/schedules. These are never serialized as content records.

`intentionallyOmittedAssets` lists verified available originals excluded by choice.
`unavailableAssets` separately lists missing/unverifiable originals. Included bytes
are digest/size verified. Source metadata, locators, attribution and existing honest
`notExtracted`/`unknown` coverage survive without originals. Import reports always
set `fullRecovery: false`, even when every original was included.

## Admission

Reader checks all wire contracts, counts, typed rows, UUIDs, duplicate IDs, foreign
keys, domain validation, exact lesson dependency closure and all asset hashes before
admission. A private database is rebuilt from bundled migrations with fixed bound
INSERT statements, default preferences and a fresh dirty search index. Synthetic
local asset registration receipts satisfy storage invariants; no source receipt is
imported and no learner attempts/history are invented. VACUUM normalizes the trusted
rebuilt file for restart verification. Only this internally built database can use
G1's private NOREPLACE staging/admission machinery; G2's public manifest and report
always retain content-only identity.

The native picker/UI integration must present the manifest's omissions and originals
choice, hold the opaque verified package and call the new-course import method.
Failed export writers may be partial; the native destination adapter must stage and
atomically publish output. No renderer filesystem authority is introduced.
