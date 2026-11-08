# Course ownership storage foundation

KER-128 under [KER-127](https://linear.app/kerigan/issue/KER-127): storage seam
implemented; startup/UI routing remains Task 2, so end-to-end migration is not
yet complete. Branch dev, base 7fc8a98; local commit only, no push/merge/release.

The user delegated migration choice. Adopt the old workspace in place as one
course; Subjects stay nested. Each new course gets a separate native-derived
root/database/originals/recovery boundary. The catalogue is private, versioned,
exclusively owned, transactional and bounded to 32 courses; exact create retries
retain identity. Missing legacy mapping, unknown data and newer versions fail
without replacing the catalogue or old workspace. Services capture course IDs
and drain before shutdown. No per-course schema migration or dependency added.

Design builder check found two ambiguities, now resolved: byte equality applies
to catalogue-only bootstrap (normal DB upgrades preserve content/identity), and
the version-1 limit refuses additional creation rather than evict pending state.

Validation: missing API RED; lost-legacy-mapping RED, then GREEN. Full Rust
library: 163 passed, 3 existing ignored. Six catalogue tests cover bytes,
restart/retry/conflict, corrupt/newer data, bound/exclusive ownership and two
real services with separate search, workspace authority and receipts. fmt and
all-target Clippy with warnings denied passed; git diff --check passed.
GUI/native visual testing, frontend changes, remote CI and tracker writes not run.

Changed: workspace/courses.rs, courses/tests.rs, mod.rs; linked design and plan.
Pre-existing AGENTS.md, .gitignore, workflow skill and November 1 reports untouched.
Next: explicit course routing across native/ports/UI (KER-129), then KER-130.

Used: Superpowers design/planning/inline execution/TDD/verification and design
builder check; engineering-workflow, tool-catalog and five Memoquill skills;
Serena, Codebase Memory Tier 2, Linear reads, local-fast extraction (provisional,
checked against source), Headroom retrieval; RTK, Git, Cargo, shell, apply_patch.
No browser, screenshots, external research, security plugin or runtime activated.
