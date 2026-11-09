# Dependency notices

2026-11-09: inventory and Cargo-lock fingerprint refreshed for M3 CSV extraction
(`csv` 1.4.0, `csv-core` 0.1.13, `ryu` 1.0.23). Actual installed notices were
collected offline, also filling nine already-locked native inventory omissions;
no existing component entry changed. The bounded collector/evidence are under
`.superpowers/sdd/2026-11-09-source-extraction/`. Rust stdlib notices are unchanged.

The Memoquill application uses Apache License 2.0 with Commons Clause 1.0; the full combined terms are in the root LICENSE. Dependencies, Rust's standard library, and adapted agent guidance retain their own licenses. This directory is shipped as passive resources in the qualification RPM alongside licenses/Memoquill-LICENSE.txt. Earlier MIT grants remain unaffected.

- `dependency-inventory.json`: exact installed Linux Cargo dependency graph and full npm lockfile inventory, license expressions, source locations and notice hashes. Build/test dependencies are deliberately included; inventory inclusion does not mean a component is linked into the application.
- `THIRD-PARTY-NOTICES.txt`: original runtime npm and Linux Cargo license/copyright texts, deduplicated by SHA-256 with per-component references and source availability links. Development/platform-only npm packages are inventoried without claiming that their files are shipped in the UI.
- `extra-sources.json` and `upstream/`: notices absent from a published crate's root, fetched from that crate's recorded VCS revision. `libappindicator-sys` uses notices from its parent package at the verified identical VCS commit. `selectors 0.38.0` ships MPL notices in its source headers; its original header is retained here and the identical full MPL 2.0 text comes from the installed `cssparser` source license.
- `rust-1.98.1/`: original Rust standard-library attribution and accompanying license files from the pinned toolchain. These are upstream legal artifacts, not Memoquill documentation or application code.

Source archives/revisions are recorded so recipients can obtain unmodified MPL-covered files; the pinned glib backport is separately documented in [the reuse record](../architecture/third-party-reuse.md). No app-wide relicensing or copyleft application code was introduced. Review the exact distribution and regenerated inventory again in M7; this inventory is evidence, not a blanket clearance of future dependencies or datasets.

Collection used `cargo metadata --locked --filter-platform x86_64-unknown-linux-gnu`, the actual installed source license files, `package-lock.json`, verified exact-revision upstream files, and the installed Rust toolchain notices. Cargo/npm lockfile SHA-256 values in the inventory detect stale dependency evidence. The task-local collection and verification scripts and raw metadata remain under ignored `.qualification/` for this review.

2026-11-01: the npm lock fingerprint was refreshed after the root project license field changed. Structural comparison with the previous lockfile confirmed every dependency entry unchanged; component licenses, notice hashes and Cargo lock fingerprint remain unchanged.
