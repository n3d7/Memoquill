# Dependency notices

The Memoquill application remains MIT. Dependencies, Rust's standard library, and the adapted agent guidance retain their own licenses. This directory is shipped as passive resources in the qualification RPM, together with the root MIT license.

- `dependency-inventory.json`: exact installed Linux Cargo dependency graph and full npm lockfile inventory, license expressions, source locations and notice hashes. Build/test dependencies are deliberately included; inventory inclusion does not mean a component is linked into the application.
- `THIRD-PARTY-NOTICES.txt`: original runtime npm and Linux Cargo license/copyright texts, deduplicated by SHA-256 with per-component references and source availability links. Development/platform-only npm packages are inventoried without claiming that their files are shipped in the UI.
- `extra-sources.json` and `upstream/`: notices absent from a published crate's root, fetched from that crate's recorded VCS revision. `libappindicator-sys` uses notices from its parent package at the verified identical VCS commit. `selectors 0.38.0` ships MPL notices in its source headers; its original header is retained here and the identical full MPL 2.0 text comes from the installed `cssparser` source license.
- `rust-1.98.1/`: original Rust standard-library attribution and accompanying license files from the pinned toolchain. These are upstream legal artifacts, not Memoquill documentation or application code.

Source archives/revisions are recorded so recipients can obtain unmodified MPL-covered files; the pinned glib backport is separately documented in [the reuse record](../architecture/third-party-reuse.md). No app-wide relicensing or copyleft application code was introduced. Review the exact distribution and regenerated inventory again in M7; this inventory is evidence, not a blanket clearance of future dependencies or datasets.

Collection used `cargo metadata --locked --filter-platform x86_64-unknown-linux-gnu`, the actual installed source license files, `package-lock.json`, verified exact-revision upstream files, and the installed Rust toolchain notices. Cargo/npm lockfile SHA-256 values in the inventory detect stale dependency evidence. The task-local collection and verification scripts and raw metadata remain under ignored `.qualification/` for this review.
