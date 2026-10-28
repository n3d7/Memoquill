# Existing applications and reusable lessons

Research snapshot: **2026-10-27**. Status: Foundation research, not implementation or permission to copy third-party code. The project remains MIT.

## Method and limits

This is a selective source inspection of four relevant projects, not a complete code audit or a claim that their UX produces learning gains. Official documentation, tagged source files, release metadata, and license files were retrieved; relevant interfaces, data definitions, comments, test cases, and notices were inspected. Larger modules were not reviewed line by line. No applications, upstream tests, importers, or benchmarks were executed. No repository was cloned and no upstream code or assets were copied into Memoquill.

Memoquill's local graph was reported as indexed at generation `2026-10-27T08:32:13Z`; the local baseline has no application implementation. External repositories were inspected through public source URLs, outside that graph. The tables below delimit the external evidence; they do not claim graph coverage or exhaustive call-chain verification.

Tags were resolved using GitHub's public API: [Anki](https://api.github.com/repos/ankitects/anki/git/ref/tags/26.09.3), [Mnemosyne tree](https://api.github.com/repos/mnemosyne-proj/mnemosyne/git/trees/2.11?recursive=1), [fsrs-rs](https://api.github.com/repos/open-spaced-repetition/fsrs-rs/git/ref/tags/v6.6.2), and [Joplin](https://api.github.com/repos/laurent22/joplin/git/ref/tags/v3.7.21). These are reproducible research references, not selected Memoquill dependency versions.

| Project | Inspected reference | Exact retrieved files; selected sections inspected | Role in this research |
| --- | --- | --- | --- |
| Anki | `26.09.3`, commit `29bb700b951e3f0c0cb69b77c0180fc1fe33e6ba` | `docs/architecture.md`; `LICENSE`; `rslib/src/storage/schema11.sql`; `rslib/src/scheduler/states/mod.rs`; `rslib/src/import_export/package/mod.rs`; `rslib/src/import_export/package/apkg/import/mod.rs`; `rslib/src/import_export/text/mod.rs` | Mature flashcard application; content/card/history separation and import boundaries. |
| Mnemosyne | `2.11`, commit `31e8f06eb8a68f91227679a1f83af58f87b15b66` | Root `LICENSE`; `mnemosyne/LICENSE`; `mnemosyne/pyqt_ui/about_dlg.py`; under `mnemosyne/libmnemosyne/`: `fact.py`, `card.py`, `component_manager.py`, `databases/SQLite.py`, `schedulers/SM2_mnemosyne.py`, `file_formats/mnemosyne2_cards.py`; `tests/test_scheduler.py`; `tests/test_anki2_import.py` | Smaller component-oriented system; scheduling edge cases and import lessons. |
| FSRS / fsrs-rs | `v6.6.2`, commit `6088949b99d44f46f5328fd7289d209b9a842b68` | `LICENSE`; `Cargo.toml`; `src/lib.rs`; `src/inference.rs`; `examples/schedule.rs` | Candidate scheduling dependency, separate from any learning application's implementation. |
| Joplin | `v3.7.21`, commit `e41516e669efb693b009bedb21f3eb1fcbea4fc9` | Root `LICENSE`; `packages/lib/models/BaseItem.ts`; under `packages/lib/services/interop/`: `InteropService.ts`, `InteropService_Importer_Raw.ts`, `InteropService_Importer_Md.test.ts`; complete repository tree metadata for license-path checks | General-purpose knowledge/import reference, not a learning-science model. |

## Anki

The [architecture note](https://github.com/ankitects/anki/blob/26.09.3/docs/architecture.md) separates a reusable library/backend from a PyQt and web-technology GUI. Python reaches Rust through a bridge; protobuf connects language layers, with the explicit qualification that the protobuf surface is not a public API. The transferable lesson is to put operations behind application services and keep views replaceable. Memoquill does not need to reproduce Anki's language layers or internal protocol.

The inspected [schema11 baseline](https://github.com/ankitects/anki/blob/26.09.3/rslib/src/storage/schema11.sql) distinguishes notes, generated cards, and review records. Cards reference their note; review records reference their card and record outcomes/timing. This is an early schema in a migration history, not a complete description of Anki 26.09.3 storage. Its useful lesson is conceptual: editable educational content, individual prompts, scheduling state, and historical attempts have different lifecycles. Memoquill should retain stable content/activity IDs and append learning events when content changes.

The [scheduler state module](https://github.com/ankitects/anki/blob/26.09.3/rslib/src/scheduler/states/mod.rs) makes new, learning, review, relearning, and filtered/preview states explicit. A context carries scheduling settings and optional FSRS next states; the file includes a boundary test for minimum/maximum intervals. This supports a pure, inspectable transition layer with clocks and randomness controlled by callers. It does not establish that four self-report buttons measure comprehension, transfer, or practical competence.

The [package modules](https://github.com/ankitects/anki/blob/26.09.3/rslib/src/import_export/package/mod.rs) distinguish Anki packages, collection packages, metadata, and media. The [APKG import entry](https://github.com/ankitects/anki/blob/26.09.3/rslib/src/import_export/package/apkg/import/mod.rs) opens an archive and delegates cards, notes, decks, and media. The [text module](https://github.com/ankitects/anki/blob/26.09.3/rslib/src/import_export/text/mod.rs) exposes CSV metadata/options and separate import handling. Memoquill should similarly treat each format as an adapter feeding a validated intermediate proposal, with duplicate policy and media/provenance handling visible before acceptance.

**Adopt as design lessons:** headless services; explicit review states; source content separate from practice history; transaction-aware import adapters; inspectable changes. **Do not adopt by default:** deck-first navigation, arbitrary executable card templates/add-ons, internal protobuf contracts, or an assumption that every learning activity is a flashcard. Anki interoperability is a later adapter, not the native Memoquill schema.

Maintenance evidence is bounded: [26.09.3 release notes](https://github.com/ankitects/anki/releases/tag/26.09.3) show scheduler/UI fixes, dependency maintenance, and test changes. This demonstrates ongoing maintenance at the inspected snapshot, not suitability of every module or a support guarantee.

## Mnemosyne

The [Fact model](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/libmnemosyne/fact.py) stores a reusable information unit; the [Card model](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/libmnemosyne/card.py) represents a question/answer view and repetition data. [SQLite storage](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/libmnemosyne/databases/SQLite.py) has distinct facts, cards, and log tables. It also documents transaction granularity being coordinated above individual writes. Memoquill should make atomic application operations explicit instead of scattering commits through UI callbacks.

The [SM2 scheduler](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/libmnemosyne/schedulers/SM2_mnemosyne.py) handles day boundaries/time zones, queue stages, sister-card spacing, interval noise, and learning ahead. Its comments explain storing retention due dates at midnight UTC and applying local/day-start adjustment when querying. The exact convention need not be reused: the lesson is to specify time semantics before implementation and test DST, travel, overdue work, repeated same-day attempts, and clock changes.

The inspected [scheduler tests](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/tests/test_scheduler.py) include timezone-controlled cases; [Anki import tests](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/tests/test_anki2_import.py) provide a concrete example of fixture-based compatibility checks. These were read, not run. Memoquill's fixtures must be created or independently licensed; upstream test data is not automatically reusable.

The [component manager](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/libmnemosyne/component_manager.py) registers components and activates component groups. That is useful evidence for clear scheduler/importer interfaces. A desktop application's in-process plugin system is not an adequate authority boundary for external agents: Memoquill should expose typed services, scopes, and reviewable proposals instead.

The [sharing format adapter](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/libmnemosyne/file_formats/mnemosyne2_cards.py) coordinates XML content, metadata, media, and ZIP packaging. Its inspected import path extracts an archive into a media directory. This is a workflow reference, not a security pattern to transplant; Memoquill must validate archive paths, expanded sizes, media types, and collisions in staging before writing accepted content. No vulnerability claim or complete security review is made here.

**Adopt as lessons:** small interfaces, reproducible temporal tests, deliberate sibling spacing, and format-specific import fixtures. **Avoid:** reproducing the SM2 implementation, allowing plugins to bypass application services, or using a format importer directly against the live library.

The [2.11 release metadata](https://api.github.com/repos/mnemosyne-proj/mnemosyne/releases/tags/2.11) records publication on **2023-11-12**; the inspected release listing still surfaced 2.11. This slower visible release cadence is a maintenance/dependency risk to investigate before adopting code, not proof the project is abandoned. Memoquill is not depending on it.

## FSRS and fsrs-rs

FSRS is a scheduling model; `fsrs-rs` is one implementation. The inspected [crate manifest](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/Cargo.toml) names package `fsrs` version 6.6.2, edition 2024, with no default feature enabled. The cost/ADR feature is explicitly experimental and may change outside normal semantic-version guarantees. It should remain out of Memoquill V1.

The [public exports](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/src/lib.rs) distinguish review data, memory/next states, evaluation, simulation, and parameter computation. The [schedule example](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/examples/schedule.rs) supplies a card's memory state, desired retention, and elapsed days to compute candidate intervals. It also illustrates why the library is not a complete application scheduler: Memoquill still owns due timestamps, time semantics, practice queues, settings, and persistence.

The inspected [inference source](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/src/inference.rs) includes tests and evaluation machinery. The presence of tests and [recent release changes](https://github.com/open-spaced-repetition/fsrs-rs/releases/tag/v6.6.2) is a useful maintenance signal, not an independent validation of Memoquill outcomes. No upstream test suite or model fitting was run.

**Recommendation:** evaluate the BSD-licensed crate behind a narrow `RecallScheduler` adapter once implementation is authorized. Start with documented default parameters and a user-visible retention/workload setting; preserve raw review events and scheduler/parameter versions. Make fitting a separate, opt-in, resource-bounded operation after enough compatible history exists. Do not silently interpret imported SM2 history as FSRS state or mix problem-solving rubric scores into recall ratings. A future scheduler replacement must not require rewriting content or deleting history.

## Joplin: knowledge and import workflows

Joplin is relevant because Memoquill also needs sources, notes, attachments, and recovery. It is not evidence about learning efficacy.

The inspected [interop service](https://github.com/laurent22/joplin/blob/v3.7.21/packages/lib/services/interop/InteropService.ts) registers format-specific import/export modules with file/directory sources and output formats. The [raw importer](https://github.com/laurent22/joplin/blob/v3.7.21/packages/lib/services/interop/InteropService_Importer_Raw.ts) maps imported IDs, recreates relationships, and rewrites internal links. [BaseItem](https://github.com/laurent22/joplin/blob/v3.7.21/packages/lib/models/BaseItem.ts) contains shared serialization/synchronization-related item handling. The lesson is that import means reconstructing a graph of content and resources, not merely inserting text rows.

The [Markdown importer tests](https://github.com/laurent22/joplin/blob/v3.7.21/packages/lib/services/interop/InteropService_Importer_Md.test.ts) exercise fixtures and imported resources. Memoquill should verify attachment/link round trips, Unicode names, duplicate imports, moved resources, interrupted imports, and deterministic loss reports. It should retain the original source even when the normalized representation loses formatting.

Official [import/export documentation](https://joplinapp.org/help/apps/import_export/) distinguishes portable formats from its JEX archive, which preserves application metadata and relationships. Its Evernote conversion discussion also makes formatting/recognition limitations visible. Memoquill likewise needs both a faithful native recovery archive and interoperable Markdown/JSON exports; a folder of Markdown files alone is not a complete personal-learning backup.

**Adopt as lessons:** module descriptors, explicit relationship remapping, resource-aware exports, and compatibility fixtures. **Do not inherit:** a note application's full sync architecture, its internal serialized representation, or its code license. The [release listing](https://github.com/laurent22/joplin/releases) supplies current maintenance evidence; no specific performance, security, or support claim follows from release activity.

## Licensing and reuse decisions

These are engineering reuse decisions based on the inspected notices, not a substitute for checking the exact shipped dependency graph. Linking/embedding a library and copying its implementation are distinct acts, but linking does not make copyleft obligations disappear. Reading ideas and independently implementing a design does not grant permission to copy code, documentation, fixtures, logos, or shared educational decks.

| Source and actual notice | Consequence for Memoquill MIT code |
| --- | --- |
| [Anki LICENSE](https://github.com/ankitects/anki/blob/26.09.3/LICENSE): AGPL version 3 or later; contributor portions BSD-3; named vendored exceptions. Inspected Rust files name **Ankitects Pty Ltd and contributors** and AGPL-3+. The logo notice names **Alex Fraser**. `docs-site` documentation has CC-BY-SA-4.0, with other files there remaining AGPL. | Architecture reference only. Do not copy/link Anki implementation into the MIT application under an assumed permissive exception. File-specific permissive notices require their own provenance check; they do not relicense the surrounding project. No logo, documentation text, deck, fixture, or asset reuse authorized. |
| [Mnemosyne root license](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/LICENSE) delegates `openSM2sync` to LGPLv3 and other code to [mnemosyne/LICENSE](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/LICENSE). The latter adds a visible Mnemosyne attribution provision to AGPLv3, with presentation to be discussed with maintainers. [About dialog](https://github.com/mnemosyne-proj/mnemosyne/blob/2.11/mnemosyne/pyqt_ui/about_dlg.py) identifies **Peter Bienstman** as main author; this is not proof of sole copyright ownership. The FSF notice in the AGPL text concerns the license document. | Architecture/test-design reference only. Do not simplify this to ordinary GPL or permissive code, and do not assume attribution alone satisfies the terms. Neither Mnemosyne nor its sync component is selected as a dependency. |
| [fsrs-rs LICENSE](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/LICENSE): BSD-3-Clause; **Copyright 2023 Open Spaced Repetition**. | Candidate dependency compatible with keeping independently written application code MIT. Retain copyright, conditions, disclaimer, and binary-distribution notices; do not imply endorsement. Audit its exact transitive dependencies when pinning. Copying an adapted file still requires its provenance and notices. |
| [Joplin LICENSE](https://github.com/laurent22/joplin/blob/v3.7.21/LICENSE): default AGPL-3.0-or-later unless a directory has its own `LICENSE`/`LICENSE.md`; notice **2016–2025 Laurent Cozic**. The [complete tree](https://api.github.com/repos/laurent22/joplin/git/trees/v3.7.21?recursive=1) showed no overriding license in the inspected `packages/lib` subtree or its `packages` ancestor. | Selected modules are references only. Do not infer MIT from historical Joplin versions or from independently licensed packages elsewhere in the monorepo. No Joplin source or fixtures copied. |

If actual reuse is later proposed, first record the exact version, source path, holder, license expression, obligations, and chosen reuse mode in [third-party-reuse.md](../architecture/third-party-reuse.md). A permissive dependency retains its own license; Memoquill's `LICENSE` must never be changed as an incidental implementation step.

## Consequences for Foundation V1

1. Model sources, editable educational content, practice prompts, scheduler projections, and immutable attempt/review events separately.
2. Use typed headless services for the GUI, import jobs, and later external-agent adapters. Importers and scheduler libraries do not receive unrestricted storage authority.
3. Make due/review behavior explicit and replayable. Recall scheduling supplements explanations, worked examples, problem solving, and practical evidence.
4. Stage imports, preserve originals/provenance, show duplicate/loss decisions, and commit accepted changes atomically.
5. Provide portable content exports and a separately versioned native backup that includes personal history and verified resources.
6. Begin with a small internal interface set. Do not implement an arbitrary in-process plugin marketplace or distributed sync to imitate mature applications.

The concrete stack recommendation and its feasibility gates are in [technical-options.md](technical-options.md). No implementation spike, dependency installation, or source-code reuse was performed by this research.
