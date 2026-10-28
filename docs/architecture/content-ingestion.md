# Content ingestion and source preservation

## Authority and handler contract

Preserve original bytes and attribution independently of extraction and pedagogical transformation. An import can succeed as a preserved attachment without claiming successful extraction. Editable lessons/activities are new revisioned objects with derivation links, not replacements for originals. AI-assisted organization uses the same proposal/application boundary as ordinary bulk edits.

Built-in handlers declare format/MIME/version, limits, extraction capability, normalized output schema, source-key strategy, warnings, and deterministic transformation version. They receive only staged selected bytes and an operation context, never broad filesystem/network authority. A handler returns normalized resource candidates and optional structured records, not executable commands. A new format adds a handler and tests; no plugin marketplace is needed.

## Pipeline and states

`selected → staged → inspected → extracted/attachment-only → mapped → validated → previewed → applied`

Each job/item also supports cancelled, failed, unsupported, duplicate, unchanged, and needs-resolution states. Preserve job/checkpoint history and reconcile item counts. Retry only failed/unapplied work; idempotency prevents duplicate application. Cancelling staging/extraction changes no live collection. User-approved partial application is a transaction over the selected valid set, with excluded items clearly reported.

1. Select files through UI/portal or bounded agent-staged upload; capture source metadata and rights status. File references alone are allowed only when explicitly chosen and labeled externally linked.
2. Copy originals into managed staging, hash raw bytes, detect MIME/encoding safely, and enforce limits. Do not rely on an extension as a trust boundary.
3. Extract/normalize in a bounded worker. Retain extraction version, text/page/block anchors and quality flags; extraction errors do not destroy originals.
4. Map structured fields and choose candidate destinations/activity templates. Preview encoding, samples, counts, duplicates, type errors, warnings, attribution, and expected changes.
5. Validate document/activity schemas and referential/prerequisite invariants. Match existing source keys and base revisions for reimport; present conflicts.
6. Commit the reviewed set through application services, with durable assets, revision/audit records, and reversible operation metadata. Index asynchronously from committed canonical records and expose indexing status.

## Initial formats

| Input | V1 behavior and fidelity |
| --- | --- |
| UTF-8 plain text/Markdown | Preserve raw file; normalize supported paragraphs/headings/lists/code/math/tables into document schema. Keep unknown syntax/source spans and show lossy conversion warnings. Do not strip Unicode or execute embedded HTML. |
| CSV | Preview dialect/encoding, column mapping, stable upstream IDs, language/sense/context, source/license fields. Generic field mapping creates resources/eligible activities; no English-only parser. Reject ambiguous rows individually with original row references. |
| JSON / Memoquill package | Accept documented versioned schemas/mapping, bounded nesting and counts. No arbitrary object-to-database deserialization. Preserve unknown supported-future payloads inertly; reject invalid references before apply. |
| PDF | Always preserve and explicitly open original. Conditional born-digital extraction/passive viewer gate in M3: maintained parser/renderer, no active content/remote loads, bounded execution, useful source anchors, licensed packaged dependencies. If it fails, ship attachment-only with clear limits. OCR is post-V1. |
| Image/media/code files | Preserve as attachments with type/size checks, captions/alt text, language hints for code, and explicit user viewing. Treat code as text. No implicit programs, macros, audio transcription, OCR, or media network retrieval. |
| EPUB / full HTML / remote documentation crawl | Post-V1 handlers. A user can import a text/Markdown export or attach the original now. No hidden web crawler. |

Do not choose an unmaintained parser simply to fill the table. M3 must compare the available Rust PDF extraction path against a packaged local PDF.js viewer/worker and select a pinned, qualified option; neither is incorporated by this planning assignment. Source preservation and typed extraction boundaries are decided now; the bounded dependency choice follows actual packaged evidence.

## Content representation and difficult text

Use versioned document JSON for authored/normalized lessons, with stable block IDs and explicit code/math/table/media nodes. Store LaTeX/code as literal source with renderer settings; no arbitrary renderer commands or link fetching. Preserve raw Unicode, language, line endings/encoding metadata and original filenames; derive normalized search text separately. Exact grading normalization must be activity-specific, never reuse search folding to silently treat different answers as equivalent.

PDF pages, source lines/JSON pointers/CSV row keys, block IDs and quote hashes provide anchors. A changed source may invalidate offsets; uncertain reattachment is visible and retains the old reference. Export both canonical document data and readable projections with fidelity reports, rather than claiming Markdown represents every source layout.

Managed originals are the default. Explicit externally linked files keep a URI/fingerprint and “external/missing/changed” state, not a false durability guarantee. A complete full backup must snapshot those files; omitted/missing external assets are enumerated and make it explicitly incomplete. Portable export can select snapshots or external-only references with declared limits. Reimport of a linked source uses explicit selection and conflict handling, never an unattended watcher overwriting content.

## Deduplication, reimport, and undo

- Exact asset SHA-256 deduplicates bytes; provenance records remain distinct. Near-duplicate text can be flagged heuristically later but never auto-merged.
- Import identity combines handler/source identity/upstream key. A changed row produces a candidate revision with a three-way comparison where possible: prior import, current human edit, new import.
- Activities/history/annotations use stable IDs and revision references. Reimport does not reset review state or replace a human-edited lesson silently.
- Undo archives/removes newly introduced placements and returns current pointers where safe. If later operations depend on the import, show a conflict/compensating preview rather than deleting new learning attempts.

## Untrusted input and limits

Use per-file/job/expanded-size limits, bounded recursion/pages/cells, cancellation/timeouts, and low worker concurrency. Proposed initial budgets for qualification: 50 MiB per ordinary file, 250 MiB per job, 500 PDF pages, 60 s worker wall time, and explicit safe override for larger user-selected jobs. These are provisional resource budgets, not format standards or guarantees.

Reject archive traversal/symlinks, malformed schemas, invalid Unicode decoding without a previewed recovery choice, active content, and executable handlers. Never run source code, PDF JavaScript, macros, shell commands, HTML scripts, or imported “instructions”. Do not fetch remote images/fonts/URLs while previewing. Markdown/HTML and math renderers have allowlists and restricted URLs. Unsafe/unsupported inputs remain preserved inertly with a clear failure state when feasible.

Fuzz parser boundaries where useful; test oversized/truncated inputs, decompression bombs, Unicode/code/math/tables, duplicate jobs, disk-full, cancellation, crash/retry, and reimport after human edits. Inspect parser dependency security advisories during implementation/release; no parser is assumed safe merely because it uses Rust.
