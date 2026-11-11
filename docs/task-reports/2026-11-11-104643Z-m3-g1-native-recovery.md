# M3 Task 2a — G1 native recovery backend

G1 backend implementation complete locally; Task 2/M3 remain incomplete.
Native file adapters/review UI and G2 portable packages are the next units.
Baseline: `3f1f227`; no push, PR, merge, release or tracker mutation in this unit.

The serialized writer captures SQLite WAL state through the backup API and copies
all referenced immutable originals before release. ZIP writing happens afterward.
The strict reader bounds EOCD/central/local metadata before ZIP allocation,
streams only declared generated names and verifies size/hash/counts, exact trusted
schema7, integrity/FKs and current organization/document/lesson domain invariants.
Missing originals stay explicit; incomplete restore requires confirmation.
New-course restore preserves internal IDs and returns the catalogue ID remap.
Durable operation intent plus a matching root-local marker prevent uncertain
root adoption; exact retries survive publication/admission and committed-reply loss.
Format, contracts and limits: [package format](../architecture/package-format.md).

RED/GREEN: missing API; real restore rejected default0755 tempfile directory;
pending intent wrongly admitted matching unowned root; invalid document JSON
with recomputed snapshot hash was accepted. Each corrected and retested.
Focused G1: 14 passed, one private crash child helper ignored by normal discovery;
the child is executed by the passing SIGKILL publication/post-commit test.
Full native: 223 passed, four private helpers ignored; native binary tests4 passed.
Offline locked Clippy all-targets with warnings denied, fmt check and diff check passed.
License inventory fingerprints and569 original notice references verified;
all original locked package/version/source identities retained.

Changed: package backend/tests, narrow course/service/database/content/lesson seams,
Cargo pins/lock, package/reuse docs and actual dependency inventory/notices.
Pre-existing agent guidance, handoff, qualification and other user files preserved.
No frontend/GUI tests were run by this unit; controller owns GUI qualification/HANDOFF.
Used: engineering-workflow, Memoquill task workflow/reporting, Superpowers TDD and
verification; Codebase Memory graph/coverage, Serena manual, Headroom retrieval,
RTK/shell, Python, Cargo/Git and reused local notice collector. No research plugin,
browser, local model, subagents, external upload or infrastructure change.
Next: independent review of this G1 unit, then typed G2/native adapter work.
