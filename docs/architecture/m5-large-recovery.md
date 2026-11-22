# M5 large workspace recovery

This design is for an explicit native recovery export and restore of a
selected course at the existing M5 fixture scale. The fixture contains
100 subjects, 10,000 resources, 100,000 activities, 1,000,000 attempts and
10 GB of managed originals. Its SQLite database alone exceeds the v1
package limit. Native v1 and portable packages keep their current formats
and admission rules.

## Measured admission inputs

The existing checkpointed v10 synthetic snapshot was measured read-only by
the verification runner. It contains 11,932,274,688 database bytes and
10,000,000,000 registered asset bytes across 10,000 originals. The largest
original is 1,000,000 bytes. The total payload is 21,932,274,688 bytes.
The database has 5,701,679 rows, including 2,100,000 study operations and
2,000,000 judgments. See
`.qualification/m5-optimized-scale/recovery-footprint.json`.

The failure mode spans database length, expanded bytes, entry count and row
limits. Keep these limits for v1. The v2 admission caps were selected to cover this measured footprint.
The per-original limit remains 64 MiB.

Capture stages the SQLite snapshot and originals on the active data
volume under the course owner lease. It checks free space before capture
and again from the selected destination file before publication. Import
checks free space before decoding and before materializing the new course.
The existing catalogue intent publishes the materialized root with a
no-replace rename. Robustness against disk pressure still depends on
handling I/O errors after these observations. Decoded staging and the
materialized course coexist until confirmation. The observed backend
roundtrip held about 62 GiB of temporary data on the SSD at its peak.

## Native v2 transport

The header starts with eight ASCII magic bytes `MQNATIVE`, the
little-endian 32-bit version `2`, and the little-endian 32-bit manifest
length. The canonical UTF-8 JSON manifest is limited to 4 MiB. Its schema,
workspace identity, byte counts and SHA-256 digests precede the raw
payloads. The database payload comes first. Originals follow in ascending
digest-path order. The decoder consumes the declared lengths, verifies
every digest and requires EOF. It creates only the known database and
canonical digest paths. Payloads use no compression.

The desktop selects v2 when native v1 capture exceeds its admission
budget. A single permit covers a large export or pending large import
review until publication, discard or acknowledged confirmation. The
capture is limited to 16 GiB of database, 24 GiB of raw payloads, 16,384
entries, 2,500,000 rows per table and 8,000,000 rows in total. SQLite
validation has an eight-minute deadline. Preserve the existing v1 and
portable ceilings.

The file contains the verified SQLite snapshot and managed originals for
one course, including private history and provenance. Export selects a
destination through the native picker. The serialized course writer pins
the snapshot and registered originals until the private copies are ready.
Publication uses those captured copies. A unique sibling staging file
is flushed and synced before atomic publication. A handled failure or
cancellation cleans staging owned by that operation.

## Restore admission and recovery

- Capture the exact supported SQLite schema and verify every payload before
  it can be activated. Reuse the current trusted DDL and canonical
  validation for preferences, documents, history, lesson revisions and
  operation receipts. Stream or batch row scans with measured bounds.
- Create a private staging root. Check destination capacity and account
  for both staging and activation before copying. Cancellation, disk
  exhaustion or concurrent ownership conflict leave the current course
  and catalogue unchanged.
- Publish the new course through the existing catalogue intent and
  no-replace recovery path. Bind a retry to the captured package identity
  and the original operation ID. Acknowledgement requires a durable
  catalogue entry. Recovery always creates an isolated course.
- Keep optional portable subsets and the v1 native format independent.
  Inspect and present inclusion and omissions before confirmation.
  Preserve linked-source provenance and refuse an incomplete restore
  unless the user explicitly accepts its declared omissions.

At startup the exclusive catalogue owner reclaims interrupted
package staging and unpublished restore staging. It retains published
course roots and durable restore intent records.

## Qualification and follow-up

- Parser regressions cover malformed framing, digest mismatch, truncation,
  trailing bytes, duplicate payloads, invalid paths and oversized manifests.
- The backend roundtrip verified the full synthetic workspace contents and
  exact retry, including every original hash and the entire database hash.
- The installed RPM exported v2 via the normal native picker. A fresh
  offline profile verified and restored that same archive. The restored
  catalogue exposes 10,000 materials. Confirmation released the review
  staging, and both course and catalogue locks released on exit.
- Retain the numeric caps and the separate human evaluation. Additional
  failure-path qualification can measure interruption during a full-size
  capture and recovery from storage pressure using small injected fixtures.

See the [qualification report](../task-reports/2026-11-21-m5-startup-recovery.md)
for the exact receipts and timing.
