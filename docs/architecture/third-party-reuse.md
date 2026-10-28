# Third-party reuse and attribution

Memoquill's project license remains MIT (`LICENSE`). Adapted material and components retain their applicable licenses. No license exception from another project applies here.

## Incorporated operational guidance

| Material | Source and holder | License | Adaptation and obligations |
| --- | --- | --- | --- |
| `AGENTS.md`; `.agents/skills/memoquill-task-workflow`; `memoquill-task-reporting`; `memoquill-tool-routing` and references | [Scryntic](https://github.com/n3d7/Scryntic), Copyright 2026 n3d7; local working copy inspected 2026-10-27 | Apache-2.0 | Adapted task/workflow/reporting/tool-routing guidance. Preserve `.agents/NOTICE`, `.agents/LICENSES/Apache-2.0.txt`, attribution, and modification notice. Actual source LICENSE/NOTICE inspected. No application code, dataset, artwork, or executable helper incorporated. |

The operational files include local-only material not all tracked in Scryntic, so a Git commit alone cannot identify their exact contents. Source hashes are recorded in `.agents/NOTICE`. The source working tree was read only. The dated machine-wide catalog, Scryntic infrastructure recipes, trading restrictions, license exceptions, and unrelated project identifiers were omitted; references were rewritten for Memoquill.

## Reuse policy

Before incorporating a dependency, copied/adapted source, dataset, documentation, media, or asset, inspect its actual license and per-file notices. Record project, source URL, authors/holder, version/commit or hash, incorporated paths, modifications, license, and obligations. Distinguish linking, vendoring, copying, and adaptation. Prefer maintained dependencies where appropriate.

Keep required license texts/notices with distributions. Evaluate source disclosure, attribution, patent, asset/data, and distribution terms independently of the MIT project license. A project license change or incompatible obligation requires explicit approval.

## Evaluated application components — not incorporated

The following are candidate reuse decisions, not an installed dependency inventory. Exact source paths/commits, holders and notices are recorded in [existing applications](../research/existing-applications.md) and [technical options](../research/technical-options.md). Do not label all shipped software MIT merely because the application is MIT.

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

Permissive dependencies can coexist with independent MIT application source while retaining their own terms. Copying/adapting source requires exact path/version, modifications, original notices and source provenance; vendoring requires tracking updates/security fixes; binary distribution requires bundled/runtime obligations. Apache-2.0 patent/NOTICE terms and BSD non-endorsement/disclaimer conditions must not be discarded. Copyleft components are not selected for embedding/copying under the current strategy; any materially different strategy needs explicit approval.

## Educational data and examples

Use small original reviewed English/Python packs with an explicit data license for initial release validation. Original English definitions/sentences must be authored independently, not paraphrased from a proprietary list. Do not bundle Oxford 5000, dictionary definitions, pronunciations, images, or audio merely because users can find them online. A user-provided import retains its stated/unknown rights metadata; private access is not automatically permission to redistribute.

Princeton WordNet 3.0 is a credible **evaluated external lexical-data candidate**, not a complete English curriculum, CEFR level list, pronunciation source, or incorporated dataset. The [publisher's actual license](https://wordnet.princeton.edu/license-and-commercial-use) permits use/modification/distribution under its conditions, retains Princeton copyright, requires preserving notices, and restricts use of the Princeton name in publicity. Verify the exact downloaded artifact's license and all included content before preparing a curated CSV; document version, selected synsets/records, transformations, attribution and review. Do not infer 3.1 or a fork's terms from this 3.0 notice. Original sample data is the simpler initial choice.

Python documentation, code examples, external exercise datasets and their solutions each need their own source/license check if redistributed. Prefer original sample exercises; no Python execution engine or third-party curriculum is incorporated here. User content, original educational data, program code, documentation, media and fonts are separate inventory categories.

## Distribution compliance plan

- Keep the root `LICENSE` for Memoquill's MIT code, `.agents/LICENSES/` and `.agents/NOTICE` for the adapted operational guidance, and `docs/architecture/third-party-reuse.md` as the human inventory.
- At implementation time generate a reviewed lockfile/SBOM-derived dependency inventory and collect exact shipped texts into a `LICENSES/` or distribution notices directory. Include notices in Linux packages and an accessible About/Legal view. Do not create a fake complete inventory before dependencies exist.
- Record component/version/commit, source URL/paths, holders, SPDX expression, reuse mode, local modifications, required notice/source/relinking/patent obligations and verification date. Flag unknowns before release.
- Audit bundled executables, webview/runtime dependencies, fonts/media and example datasets independently; a package-manager dependency differs from vendored binary distribution. Check exact release artifacts and advisories, not just repository popularity.
- No project license change, incompatible source incorporation, commit, push or redistribution occurred during this assignment. This inventory is an engineering compliance plan, not a legal determination for every future dependency combination.
