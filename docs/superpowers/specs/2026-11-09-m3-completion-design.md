# M3 completion: ingestion, discovery and recoverable ownership

User authority (2026-11-09): finish all M3, commit completed substeps on dev,
publish dev and create a correctly described dev-to-main PR; do not merge.
This supersedes earlier local-only/no-commit continuation wording. Keep unrelated
tooling edits out of commits. Preserve original history; no force-push/rebase.

## Design

Build on saved course-owned originals, the typed writer service, exact receipts,
immutable document/lesson revisions and the completed passive preview slice.
Use one SQLite migration for durable ingestion records and a rebuildable FTS5
index; use native snapshot/archive adapters for ownership. UI routes remain
course-bound and passive. No external-model transport, schedules or M4 learning
history is introduced; content edits retain all available history/revision pins.

### Ingestion and indexed discovery (KER-100)

An explicit saved-original selection creates a durable job with handler version,
source ID/digest/revision, extraction coverage, mapping, item locators/keys,
candidate values, diagnostics and outcomes. TXT/Markdown remain literal text,
CSV has selectable field mapping/no implicit headers, JSON accepts documented
raw records/mapping rather than arbitrary database deserialization. Limits,
validation errors and unsupported inputs are explicit, and originals remain usable.
Actual apply operates only on complete validated reviewed records; prefixes from
bounded previews never implicitly import the unseen file. Empty and omitted items
reconcile visibly. User-approved partial apply enumerates the selected subset.

Jobs survive restart. Cancel changes no live material before apply. Apply/retry
has durable exact outcomes and no silent duplicates. Reimport explicitly selects
a prior import lineage, compares prior imported values/current human values/new
source values and presents unchanged/new/changed/conflict/removed or unresolved
items. Human edits and uncertain anchors are never silently overwritten; explicit
resolution is separate from default apply. Undo previews compensation, refuses
stale/dependent reversal, and preserves immutable revisions/annotations/history.
All requests validate expected revisions, authority, original digests and bounds.

Search uses rebuildable FTS5 over committed canonical title/text with stable IDs,
course-local results and attributable original/range references. Preserve existing
pagination/query validation semantics, including Unicode/code and short queries.
Index corruption/staleness is visible/recoverable, never source authority.

### Format qualification

Maintain an actual offline view/extract matrix for TXT/Markdown/CSV/JSON,
PDF/EPUB/DOCX/HTML and relevant attachments. Qualify parser maintenance/license,
passivity, expansion/depth/time/memory and packaged availability before enabling
it. No active HTML/XML entities/macros, remote fetch or arbitrary executable files.
If a parser fails or remains unqualified, preserve original-only capability and
explicit failed/unsupported/partial coverage. PDF gate must have an explicit
evidence-backed result; EPUB is a qualification target, not silently excluded.
Manual access to originals must be useful offline, with explicit user-selected
copy/open actions behind native adapters; no renderer-supplied paths.

### Packages and recovery (KER-101)

Use versioned ZIP/JSON/NDJSON manifest/sections with safe relative names, lengths,
hashes, declared kind/subset/completeness and future history adapter versions.
G1 private full backup uses SQLite backup while serialized against the writer,
and leases/captures every referenced immutable asset before releasing the barrier.
Never copy only a live WAL main file. Include all current canonical IDs/revisions,
sources/locators/jobs, lessons/bindings, preferences/drafts/resume and history
available now. Unknown required future sections/schema fail safely.

G2 portable package declares shareable content/source subsets and readable
projections with omission/remap reports; content-only is never full recovery.
No implicit private-original sharing or credentials/grant/agent-token export.
Restore validates all metadata/archive bounds/names/duplicates/symlinks/hashes,
SQLite integrity/schema and references in staging, then creates a NEW course;
failure leaves every live course unchanged. No destructive replacement workflow.
Missing originals make backup explicitly incomplete; incomplete archives cannot
claim a verified full restore. Preserve external-link incompleteness honestly.

## Completion evidence

Automated tests cover real writer/restart/retry/cancel/partial/reimport/manual-edit/
undo conflicts, Unicode/code search scale/rebuild/scope, active-WAL DB/assets
snapshots, corruption/future-schema/path/bomb/symlink and failed restore rollback,
G1/G2 inclusion/omission/round trips. UI behavior covers keyboard-friendly controls,
status/review/errors and course switching; packaged X11/headless qualification is
distinct from unrun human speech/reflow M7 acceptance. GUI requires user approval;
agents never capture screenshots. Do not call M3 complete with missing software
acceptance; explicitly record any remaining human-only M7 boundaries.
