# M3 first slice: bounded source extraction preview

The user approved starting M3 in the proposed order on 2026-11-09. This slice
extends existing saved-original previews with passive versioned extraction;
it does not complete KER-100. Implement inline in the existing dev workspace,
preserving unrelated changes and the existing no-commit/no-publication boundary.

## Approach and ownership

Reuse the course-bound source service, verified asset read and preview command.
Adding a separate file-reading endpoint would duplicate authority; parsing in the
browser would duplicate native policies. Keep parsing in a focused native module
and render its typed result as ordinary text in the existing source panel.
No database migration is required: derived previews are recomputed, not durable
import jobs. Full job/apply/reimport/index contracts follow this slice.

## Contract

Add optional `extraction` to SourcePreview for compatibility with older consumers.
Production native previews always supply it. Version 1 names format (text,
markdown, csv, json or unsupported), status (complete, partial, attachmentOnly,
failed), bounded records, and a typed optional diagnostic. Each record has a
locator kind/value and string fields; envelope source ID/digest pins the original.
Complete describes extraction only, never reading/learning or successful import.

Use the existing 65,536-byte preview bound, at most 128 displayed records and
64 fields per record. Truncated structured inputs are not parsed as whole files;
return partial/inputLimit with no misleading successful JSON/CSV record. Text
can expose a partial prefix. Invalid UTF-8 is failed/invalidUtf8. Unknown formats
remain attachmentOnly/unsupportedFormat, including HTML/PDF/EPUB/DOCX until their
separate qualification. File extension selects a candidate handler, not trust;
UTF-8 and format parsing validate content. No heuristic binary-to-text guessing.

Text/Markdown expose source lines; Markdown stays literal, without HTML rendering
or link fetching. CSV uses the maintained csv crate, fixed comma dialect,
no assumed header and no trimming; records preserve quoted/multiline fields.
Unequal field counts fail visibly; column mapping/dialect choice follows later.
JSON validates with serde_json's default depth limit and preserves each top-level
array item's raw JSON, or one root JSON value, with JSON Pointer locators. Raw
record preservation prevents duplicate object keys silently disappearing.

All parser input/output is bounded. Full asset verification, authorization and
course routing remain before extraction; extraction cannot mutate content,
original bytes, receipts or history, execute code, open URLs or gain file access.
Dependency licenses/notices are recorded when incorporating csv 1.4.0.

## Acceptance and remaining M3 scope

Tests exercise Unicode/CRLF/BOM, quoted CSV/newlines/empty fields, JSON pointers,
raw duplicate keys, malformed/oversized/deep/binary inputs, record/field caps,
restart-stable source previews and unchanged bytes/revisions. UI tests assert
visible coverage/diagnostics/locators and inert hostile strings.
Normal build/frontend/native gates and independent final review precede delivery.
No GUI launch/screenshots, external uploads or scientific efficacy claims.

Remaining: durable import job/reconciliation, mapping/review/apply, conflict-aware
reimport/undo, qualified document parsers and rebuildable scoped indexing; KER-101
portable package/recovery is separate. Engineering bounds above are chosen for
this preview, not a reduction of the later ingestion budgets or format scope.
