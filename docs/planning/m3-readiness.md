# M3 ingestion: continuation readiness

Checked 2026-11-09 against local dev `f80840d`, source and Linear.
[KER-100](https://linear.app/kerigan/issue/KER-100) remains Backlog.
The user approved starting in this order on 2026-11-09. The first passive
extraction slice is defined by its [design](../superpowers/specs/2026-11-09-source-extraction-design.md)
and [plan](../superpowers/plans/2026-11-09-source-extraction.md); the original
prerequisite evidence below is retained as a dated baseline.

## Completed baseline

- Linear reads KER-99 and KER-127/128/129/130 Done. The latter descriptions
  retain historical planning language; current code and completion report matter.
- Course foundation commits: `87648ff`, `c6a363e`, `f80840d`.
  [Runtime contract](../architecture/course-workspaces.md) preserves existing data
  as one course and binds services/state to course identity.
- Original-byte publication and exact receipts already have a restart/dedup test:
  `source_bytes_publish_once_and_receipt_survives_restart` in
  `src-tauri/src/workspace/assets.rs:225`.
- `Database.search_resources` in `src-tauri/src/workspace/search.rs:26`
  currently lowercases the query and scans bounded rows. Reuse the public scoped
  service; this function does not itself establish M3 FTS5 readiness.
- Fresh checks: 140 frontend tests/26 files; Rust library 173 passed/3 ignored.
  These checks do not qualify a packaged course journey or human accessibility.

## Reconciled requirements

The [ingestion architecture](../architecture/content-ingestion.md) now reflects
the later KER-100 clarification: EPUB is an M3 qualification target rather than
blanket post-V1 scope. DOCX/HTML/additional formats require a view/extract matrix,
not a promise of universal parsing. Developer Docling/MarkItDown availability
does not imply product support or an approved runtime dependency.

Sources belong to a course, with explicit lesson bindings, immutable originals,
metadata, extraction revisions, used ranges and honest partial/unread coverage.
Intake stays offline and useful without AI. No implicit folder traversal authority,
network fetching, active content execution or external-model upload is granted.

## Proposed order for the next implementation design

1. Define versioned intake job/item/handler and extraction/locator contracts over
   existing originals, scoped services, revisions and receipts. Include recovery,
   cancellation, item reconciliation and resource budgets before parser choices.
2. Deliver TXT/Markdown/CSV/JSON preview, mapping and reviewed apply; verify
   exact retry and unchanged reimport without duplicate content/provenance loss.
3. Add changed-source conflict/undo previews preserving human edits, annotations
   and historical revisions. Never silently merge uncertain anchors.
4. Qualify PDF/EPUB/DOCX/HTML view/extract capability with licensed, maintained,
   passive packaged dependencies; retain usable originals on failure.
5. Deliver rebuildable scoped indexing with stable source/range IDs and explicit
   indexing state; exercise Unicode/code and beginning/middle/end at scale.

This order was accepted by the user; the first preview slice is not a reduced M3 scope.
KER-101 portable packages/full recovery remains the separate M3.2 deliverable.
GUI launch needs explicit approval; screenshot capture is prohibited.

## Evidence limits and next action

Graph Tier 2, project `home-void-Project-Memoquill`, generation
`2026-11-09T09:40:47Z`: bounded searches, search-service inbound/outbound trace,
exact search/asset-test snippets. Six relied-on code paths have matching coverage
metadata and no recorded issues; this is best-effort, not an exhaustive audit.
No claim that all ingestion functionality is absent follows from these searches.
Local `origin/dev` and dev match; no network fetch/remote publication check ran.
No build/security/parser qualification or new scientific research ran.

Next after the first preview slice: durable intake jobs and reviewed mapping/apply,
then remaining reimport/parser/indexing acceptance. Retain Linux/X11 scope and
deferred M7 speech/reflow acceptance.
