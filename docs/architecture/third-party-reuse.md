# Third-party reuse and attribution

## G1 native package implementation (2026-11-11)

Pinned `zip`9.5.0 is linked with default features disabled and deflate enabled.
Its installed MIT text retains Mathijs van de Nes copyright2014; source
archive/checksum and complete license text are in the dependency inventory/notices.
The selected version is newer than the corrected RustSec advisory ranges in the
task dependency evidence; application EOCD/metadata, path, overlap and expansion
limits remain necessary. No ZIP implementation is copied or vendored.
`tempfile`3.27.0, already pinned for tests, is promoted to runtime for explicitly
0700 private staging. New Linux notice entries cover zip, zopfli, typed-path,
constant_time_eq, zeroize, zeroize_derive, bumpalo and rustversion, retaining their
installed original license texts. Original dependency versions are preserved;
refreshed lock edges include platform feature unification. Collector and bounded
metadata evidence are in `.superpowers/sdd/2026-11-09-m3-completion/`.

Memoquill's project license is Apache License 2.0 subject to Commons Clause 1.0 (`LICENSE`), adopted on 2026-11-01 at the user's explicit request. The combined terms are source-available, not OSI-approved open source. Earlier MIT grants are not revoked; separately licensed dependencies and adapted guidance are not relicensed by this change. Adapted material and components retain their applicable licenses. No license exception from another project applies here.

## Incorporated operational guidance

| Material | Source and holder | License | Adaptation and obligations |
| --- | --- | --- | --- |
| `AGENTS.md`; `.agents/skills/memoquill-task-workflow`; `memoquill-task-reporting`; `memoquill-tool-routing` and references | [Scryntic](https://github.com/n3d7/Scryntic), Copyright 2026 n3d7; local working copy inspected 2026-10-27 | Apache-2.0 | Adapted task/workflow/reporting/tool-routing guidance. Preserve `.agents/NOTICE`, `.agents/LICENSES/Apache-2.0.txt`, attribution, and modification notice. Actual source LICENSE/NOTICE inspected. No application code, dataset, artwork, or executable helper incorporated. |

The operational files include local-only material not all tracked in Scryntic, so a Git commit alone cannot identify their exact contents. Source hashes are recorded in `.agents/NOTICE`. The source working tree was read only. The 2026-10-28 revision restores useful workstation tool commands, resource boundaries, Context7/Firecrawl routes, document/MCP diagnostics, verification and handoff detail in Memoquill's skill references. The dated machine-wide catalog, product infrastructure/monitoring recipes, trading restrictions, license exceptions, and unrelated project identifiers remain omitted. Shared lab service instructions are bounded to authorized Memoquill targets. No executable helper or application code is copied by this guidance update; root licensing is unchanged.

## Reuse policy

### M3 passive CSV preview (2026-11-09)

Linked registry dependencies `csv` 1.4.0 and `csv-core` 0.1.13 are unmodified
Andrew Gallant components from [rust-csv](https://github.com/BurntSushi/rust-csv),
dual Unlicense/MIT; actual COPYING and MIT texts were inspected. `ryu` 1.0.23
by David Tolnay is Apache-2.0 OR BSL-1.0. Exact source archives, license texts,
holders and notice hashes are included in the refreshed
[inventory](../licenses/dependency-inventory.json) and
[notices](../licenses/THIRD-PARTY-NOTICES.txt), already configured as packaged
resources. Preserve the original notices under their own terms.
The handler uses [ReaderBuilder](https://docs.rs/csv/1.4.0/csv/struct.ReaderBuilder.html)
over bounded bytes with no assumed headers; strict quote validation precedes the
library's permissive recovery. No upstream source was copied or modified.

The inventory refresh also captures nine already-locked native dependencies
missing from the older inventory; existing component entries remain identical.
This does not add those dependencies or claim a new dependency security audit.

Before incorporating a dependency, copied/adapted source, dataset, documentation, media, or asset, inspect its actual license and per-file notices. Record project, source URL, authors/holder, version/commit or hash, incorporated paths, modifications, license, and obligations. Distinguish linking, vendoring, copying, and adaptation. Prefer maintained dependencies where appropriate.

Keep required license texts/notices with distributions. Evaluate source disclosure, attribution, patent, asset/data, and distribution terms independently of the combined project license. A project license change or incompatible obligation requires explicit approval.

## Evaluated application components — not incorporated

The following are candidate reuse decisions, not an installed dependency inventory. Exact source paths/commits, holders and notices are recorded in [existing applications](../research/existing-applications.md) and [technical options](../research/technical-options.md). Do not apply the application license to third-party components.

| Project/component | Inspected reference and license | Decision and obligations |
| --- | --- | --- |
| Anki, Ankitects Pty Ltd and contributors | 26.09.3; AGPL-3.0-or-later with file/vendored exceptions; separate artwork/docs terms | Design/source reference only. No implementation, fixtures, decks, docs text, or assets copied. Do not assume linking avoids copyleft or one permissive file relicenses a subsystem. |
| Mnemosyne, Peter Bienstman/main-author attribution plus other contributors | 2.11; inspected application AGPLv3 text includes an additional visible-attribution provision; sync component has separate LGPL terms | Reference only. Do not treat the extra provision as permission to copy under MIT; discuss exact terms before any future reuse. |
| Joplin, Laurent Cozic/contributors | 3.7.21; inspected selected modules under default AGPL-3.0-or-later | Import/ownership design reference only. Historical permissive versions and differently licensed subpackages do not license these inspected modules. |
| fsrs-rs, Open Spaced Repetition | v6.6.2, BSD-3-Clause | Preferred recall dependency candidate. Preserve copyright/conditions/disclaimer and binary notices; no endorsement. Audit the pinned transitive graph. No fitting or experimental cost/ADR feature required. |
| Tauri | stable 2.12.5 research reference, Apache-2.0 OR MIT; distinct contributor/Commons Conservancy notices | Candidate linked framework. Preserve chosen applicable license/notices; WebKitGTK/system libraries and bundled tools have independent compliance obligations. |
| React / ProseMirror | inspected reference modules MIT; Meta/affiliates and Marijn Haverbeke/other contributors respectively | Candidate UI/editor dependencies. Verify every selected package/plugin; do not infer an entire editor ecosystem's terms from one module. |
| SQLite | official public-domain dedication for delivered SQLite; build tools/bindings separate | Candidate embedded database. Record exact bundled library, Rust binding and extension notices. |
| Official Rust MCP SDK (`rmcp`) | inspected commit `cd88056ffdc97756fbd98e5a12bbeb41758f0ce8`; manifest 3.8.0; actual LICENSE describes MIT→Apache-2.0 transition with some earlier MIT contributions and separately licensed non-spec docs | Preferred protocol adapter candidate; verify a released pin and preserve applicable per-file/history notices. Cargo metadata alone is insufficient. No SDK copied/installed now. |
| PDF.js / Rust PDF parser, KaTeX, Radix, `rusqlite`, testing/build tools | Implementation candidates; exact selected artifacts/transitives not yet audited | No blanket license approval. M1/M3/M7 verify actual license/security/runtime requirements before incorporation. PDF processing remains conditional. |

Permissive dependencies can coexist with independently licensed application source while retaining their own terms. Copying/adapting source requires exact path/version, modifications, original notices and source provenance; vendoring requires tracking updates/security fixes; binary distribution requires bundled/runtime obligations. Apache-2.0 patent/NOTICE terms and BSD non-endorsement/disclaimer conditions must not be discarded. Copyleft components are not selected for embedding/copying under the current strategy; any materially different strategy needs explicit approval.

## Educational data and examples

Use small original reviewed English/Python packs with an explicit data license for initial release validation. Original English definitions/sentences must be authored independently, not paraphrased from a proprietary list. Do not bundle Oxford 5000, dictionary definitions, pronunciations, images, or audio merely because users can find them online. A user-provided import retains its stated/unknown rights metadata; private access is not automatically permission to redistribute.

Princeton WordNet 3.0 is a credible **evaluated external lexical-data candidate**, not a complete English curriculum, CEFR level list, pronunciation source, or incorporated dataset. The [publisher's actual license](https://wordnet.princeton.edu/license-and-commercial-use) permits use/modification/distribution under its conditions, retains Princeton copyright, requires preserving notices, and restricts use of the Princeton name in publicity. Verify the exact downloaded artifact's license and all included content before preparing a curated CSV; document version, selected synsets/records, transformations, attribution and review. Do not infer 3.1 or a fork's terms from this 3.0 notice. Original sample data is the simpler initial choice.

Python documentation, code examples, external exercise datasets and their solutions each need their own source/license check if redistributed. Prefer original sample exercises; no Python execution engine or third-party curriculum is incorporated here. User content, original educational data, program code, documentation, media and fonts are separate inventory categories.

## Distribution compliance plan

- Keep the root `LICENSE` for Memoquill's application code and combined terms, `.agents/LICENSES/` and `.agents/NOTICE` for the adapted operational guidance, and `docs/architecture/third-party-reuse.md` as the human inventory.
- At implementation time generate a reviewed lockfile/SBOM-derived dependency inventory and collect exact shipped texts into a `LICENSES/` or distribution notices directory. Include notices in Linux packages and an accessible About/Legal view. Do not create a fake complete inventory before dependencies exist.
- Record component/version/commit, source URL/paths, holders, SPDX expression, reuse mode, local modifications, required notice/source/relinking/patent obligations and verification date. Flag unknowns before release.
- Audit bundled executables, webview/runtime dependencies, fonts/media and example datasets independently; a package-manager dependency differs from vendored binary distribution. Check exact release artifacts and advisories, not just repository popularity.
- The original foundation research made no license change. On 2026-11-01 the user authorized the combined Apache-2.0 / Commons Clause project license; unrelated third-party terms remain intact. This inventory is an engineering compliance plan, not a legal determination for every future dependency combination.

## Project license metadata (2026-11-01)

The complete combined text is in the root `LICENSE` and is bundled in RPM resources as `licenses/Memoquill-LICENSE.txt`. Cargo uses `license-file`; npm uses `SEE LICENSE IN LICENSE`; the desktop bundle uses the local identifier `LicenseRef-Memoquill-Apache-2.0-Commons-Clause-1.0` plus `licenseFile`. None of these labels claims an SPDX-listed Commons Clause exception or plain Apache-2.0 licensing.

Texts were retrieved from [Apache](https://www.apache.org/licenses/LICENSE-2.0.txt) and the [Commons Clause publisher](https://commonsclause.com/). The condition identifies Memoquill, Apache License 2.0, and licensor n3d7. Only those three template fields were filled; the Apache text is unchanged. The publisher explains the commercial restriction and that existing open-source grants are not revoked. Existing per-file Apache notices in adapted operational guidance and third-party dependency notices remain applicable.
