# Source format capability and qualification gates

This matrix distinguishes preserving/viewing original bytes, extraction, creating
editable material, and any claim about actual reading. An attachment, extraction
success or imported lesson is never evidence that the learner or an agent read
an entire book. Pending cells are filled only after implementation qualification.

| Format | Preserved original / passive preview | Extraction/import capability | Qualification state |
| --- | --- | --- | --- |
| UTF-8 TXT | Immutable bytes; bounded literal text | Durable source-line intake with locators and reviewed bounded apply | Task 1 backend tests/review accepted; final integrated qualification pending |
| UTF-8 Markdown | Immutable bytes; literal markup without executing HTML/links | Durable literal-line intake; raw syntax remains attributable, no universal layout conversion claim | Task 1 backend tests/review accepted; final integrated qualification pending |
| CSV | Immutable bytes; explicit no-header-assumption preview | Fixed comma dialect, strict quotes, multiline fields; explicit key/title/body mapping and reviewed apply | Task 1 tests/review accepted; native CSV/reimport subset passed, final candidate pending |
| JSON | Immutable bytes; raw top-level values/pointers | Versioned raw-record and named-property mapping; duplicate properties remain unresolved, no arbitrary DB object loader | Task 1 tests/review accepted; native named-property subset passed, final candidate pending |
| PDF | Preserve/copy original; raw/binary preview | No qualified in-app parser/viewer incorporated | Original-only fallback; final preservation/access tests pending |
| EPUB | Preserve/copy original container | No qualified EPUB/XML/XHTML extractor incorporated | Explicit M3 target: original-only fallback tests pending |
| DOCX | Preserve/copy original container | No qualified OpenXML converter incorporated | Original-only fallback tests pending |
| HTML | Preserve original; tags displayed literally as text | No DOM, scripts, embedded resources, automatic crawling or renderer execution | Passive-text/original-only tests pending |
| Other document/image/media/code | Preserve admitted bounded original with attribution | Type-specific rendering/extraction unqualified; code is never executed | Binary/text fallback tests pending |

## PDF gate

Candidate evidence reviewed 2026-11-09: [PDF.js official architecture and build
layout](https://mozilla.github.io/pdf.js/getting_started/) separates parser worker,
display and viewer and ships accompanying fonts/maps/resources. Its maintained
distribution is a candidate, not installed Memoquill capability. No PDF parser,
worker limits, passive viewer, dependency security result or packaged RPM behavior
has been qualified in this application. Gate result: **NOT QUALIFIED; original-only**.
This chooses KER-100's explicit fallback and does not record a passed extraction
test or allege an upstream defect. Actual original copy/access and honest statuses
must pass the final M3 fixture journey before closing the acceptance row.

## EPUB and OpenXML/HTML gate

[EPUB 3.3](https://www.w3.org/TR/epub-33/) packages web resources inside an OCF ZIP
container. Recovery ZIP support alone is not EPUB parsing or a reading system.
Any later converter must constrain archive expansion/nesting, validate references,
disable entities/macros/scripts and remote resource fetching, and return useful
locators/coverage. Those converters are unqualified here, so original-only fallback
is explicit. Developer Docling/MarkItDown tools do not establish product support.
No external-source content is uploaded as part of this decision.

## Fixture evidence and resource limits

Original synthetic inputs are under `tests/fixtures/m3/`. They include a PDF,
EPUB/DOCX containers, traversal/entity EPUB and hostile HTML. Deterministic
regeneration and PDF metadata validation passed at fixture preparation; application
preservation/access/no-execution tests are tracked in
[M3 qualification](../quality/m3-qualification.md). These fixtures do not claim
universal format conformance or extraction quality.

The first preview limits remain 64 KiB / 128 records / 64 fields. Actual intake
has separate explicit validated limits and cannot apply an unseen prefix.
Cancellation, malformed/binary input, unsupported format, partial coverage and
original-only fallback must stay distinguishable in UI, jobs and package reports.

## Final M3 qualification — 2026-11-13

The final external full runner and isolated native full journey passed. CSV/JSON intake, reimport/undo and native search were exercised live; TXT/Markdown remain covered by the full headless handler suite. Original CSV/PDF/EPUB/DOCX/hostile EPUB/HTML copies preserve exact names and SHA-256 bytes. PDF, EPUB and DOCX extraction/viewing remain NOT QUALIFIED; their original-only fallback is qualified. HTML remains inert text/original access with no automatic resource fetching or execution. Unknown additional formats are not universally qualified. See the final receipt in [M3 qualification](../quality/m3-qualification.md), which supersedes earlier pending fixture/access cells without changing parser gates.
