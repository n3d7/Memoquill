# M3 first slice: passive source extraction preview

2026-11-09 11:03 UTC. User approved starting M3 in the proposed order. First
slice complete locally; [KER-100](https://linear.app/kerigan/issue/KER-100)
remains incomplete. No new tracker read/write in this slice; earlier Backlog
readback retained. Branch dev, HEAD f80840d; no commit/push/merge/CI/release.

Saved originals now supply a version-1 derived preview for TXT/Markdown lines,
CSV records and raw JSON values, with locators and honest complete/partial/
attachment-only/failed outcomes. Bounds: 64 KiB input, 128 records, 64 fields.
Parsing and JSX are passive. Unqualified PDF/EPUB/DOCX/HTML keep original-only
fallback. Existing course-bound command, asset verification, bytes, receipts and
history are retained; no database migration or parser filesystem/network authority.

Changed: native source types/preview/new extraction handlers and tests, source UI
and DTO/tests, Cargo manifest/lock; design/plan, roadmap/readiness/handoff and
dependency inventory/notices/reuse docs. Pre-existing global tooling changes,
reports and earlier continuation documentation preserved.

RED/GREEN: missing extractor and missing preview field; UI missing five outcomes;
malformed CSV quote fixture returned Complete before strict syntax validation.
Final checks: frontend 145 passed/26 files, native library 182 passed/3 existing
ignored, startup 4 passed, doctests 0; build/fmt/Clippy/diff pass. Coverage 80.63%
statements, 71.37% branches, 78.72% functions, 92.22% lines. Existing Vite large
bundle warning persists. No GUI/screenshots, RPM, speech/reflow, new security scan
or scientific efficacy evaluation; unchanged release GLib evidence not rerun.

Independent read-only review found no Critical/Important/Minor issues. Reviewer
excluded full KER-100 and real GUI acceptance from this slice verdict; both remain
unclaimed. No deferred minor findings. No other material review rulings.

Decisions: existing dev workspace retained; inline TDD then independent review;
offline Cargo cache used after network DNS failure. csv 1.4.0/csv-core 0.1.13/ryu
1.0.23 added with actual notices. Inventory refresh also filled nine already-locked
native omissions; existing entries unchanged, both lock fingerprints verified.
Evidence: .superpowers/sdd/2026-11-09-source-extraction/, /tmp/memoquill-m3-*.log.
Ledger retained because work has no Git commit record. No background jobs remain.

Next: durable intake jobs/item reconciliation, reviewed mapping/apply, then changed
source conflicts/undo, document qualification and indexing. KER-101 remains separate.
Used: engineering-workflow, Memoquill workflow/context/reporting, Superpowers plan/
execution/TDD/debugging/verification/review/finishing skills; MCP Serena, Codebase
Memory, local_ai and Headroom; RTK/Git/Python/npm/Vitest/Cargo, apply_patch, clock,
web official csv docs and one reviewer agent. No runtime plugin used in this slice;
Context7 schema only discovered, not called. No external private content transmitted.
