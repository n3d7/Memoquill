# Architecture decisions

Status: **recommended for Foundation V1**, with explicit implementation-time qualification gates. ADR-02's M1.1 candidate has passing bounded native automation and remains subject to manual desktop/Orca acceptance and user review; ADR-08 is applied locally. This is not release qualification or a learning/performance benchmark. Material reconsideration updates this record and the roadmap rather than quietly adding a second architecture. See the [actual Fedora baseline](../quality/fedora-desktop-baseline.md).

## ADR-01 — A local Linux modular monolith

- **Context/requirements:** P01–P14 require useful offline personal learning, ownership, durability, and optional AI; V1 targets Linux/Fedora only.
- **Alternatives:** cloud/web-first service; distributed local services; one desktop process with modules and bounded workers.
- **Evidence/trade-offs:** no collaboration/sync requirement justifies operational infrastructure. A single application owner simplifies transactional consistency and support, though UI/process failure affects the workspace until recovery.
- **Decision:** one desktop application owns SQLite/assets and application services; no account, mandatory cloud, or permanent daemon. Isolated parser jobs and stdio bridges are bounded adapters.
- **Consequences:** direct typed module calls, offline qualification, explicit single-instance/workspace locking. Platform adapters preserve a future route to portability without implementing other platforms.
- **Reconsider when:** a validated use case genuinely requires headless multi-client operation, sync, or collaboration; do not infer it from MCP support alone.

## ADR-02 — Rust + TypeScript/React + stable Tauri 2

- **Context/requirements:** maintainable desktop integration, mature UI/content tooling, accessibility, and reasonable deployment effort.
- **Alternatives:** Electron/TypeScript, GTK4/libadwaita/Rust, Qt, Tauri 3 alpha. See [technical comparison](../research/technical-options.md) for source-backed details.
- **Evidence/trade-offs:** Tauri 2 reuses WebKitGTK and integrates Rust without a server; Electron bundles its rendering runtime at a footprint cost; native GTK offers strong Linux integration but more bespoke rich educational editing. No Memoquill performance comparison exists. Tauri 3 alpha does not justify migration risk now.
- **Decision:** stable Tauri 2/Rust core, React/TypeScript interface, app-owned document schema with a mature editor adapter. Pin dependencies in M1 after Fedora packaged checks.
- **M1.1 disposition (2026-10-28):** retain the pinned Tauri 2 candidate. Offline RPM installation, 35 interaction and 7 Orca-presentation assertions per X11/native-Wayland mode pass in the recorded Fedora 44 profile. Important results use an explicit, reversible dialog; automatic polite notifications are not qualified. Physical GNOME/hardware and human listening remain manual acceptance, so this is not unconditional release approval. KER-97 goes to review; KER-98 does not start here.
- **Consequences:** WebKitGTK/AT-SPI/IME/Wayland behavior is a critical release risk, not solved by a framework claim. M1 must qualify keyboard/Orca/editor/math before expanding the domain.
- **Reconsider when:** core workflows fail the bounded accessibility/runtime gate; prefer an evaluated Electron or native alternative rather than accumulating fragile workarounds. Revisit Tauri 3 after stable release and relevant evidence.
- **Bounded fallback:** isolate a failing core case once, distinguish application from runtime behavior, then compare the same fixture in Electron and GTK4 if unresolved. Record deployment/accessibility cost before changing the decision; no concurrent second stack or custom Orca fork is approved.

## ADR-03 — Familiar containment plus typed relationships

- **Context/requirements:** P01/P02/P10 and four-subject cases require arbitrary organization, shared resources, prerequisites, and alternate paths.
- **Alternatives:** fixed Subject→Module→Topic tree; unrestricted graph as primary UI; collection tree plus shared references and typed edges.
- **Evidence/trade-offs:** a tree is recognizable but fixed depth/ownership duplicates shared knowledge. A graph-first editor adds navigation and maintenance burden with no demonstrated V1 benefit.
- **Decision:** arbitrary-depth collections and placements, units/objectives/resources with stable IDs, prerequisite DAG and cyclic related links, ordered paths, tags/search. Module/topic are labels, never special subject tables.
- **Consequences:** validate containment/prerequisite cycles and ownership; moving a placement cannot alter content/history identity. No graph DB/canvas.
- **Reconsider when:** observed collection tasks require genuinely multiple collection parents or specialized graph navigation; first test views/references rather than changing the domain to match one curriculum.

## ADR-04 — SQLite, immutable revisions, and preserved assets

- **Context/requirements:** P02/P04/P06/P12 require durable attempts, reimport, undo, search, and recoverability.
- **Alternatives:** directory-only Markdown; document database; relational data with unrestricted JSON; relational identities/history plus versioned content payloads/assets.
- **Evidence/trade-offs:** SQLite provides transactions, constraints and FTS5, but filesystem assets require an explicit consistency protocol. Editor-native/Markdown-only formats cannot faithfully represent all sources or activity contracts. [SQLite constraints](https://sqlite.org/foreignkeys.html), [backup](https://sqlite.org/backup.html), [FTS5](https://sqlite.org/fts5.html).
- **Decision:** relational identities/relationships/history, immutable revision snapshots, validated versioned document/activity JSON, content-addressed raw assets, rebuildable indexes, one writer.
- **Consequences:** storage grows with revisions; garbage collection must respect references/backups. No generic event-sourcing framework, second search database, or live-file-copy backups.
- **Reconsider when:** measured collection/query limits exceed SQLite/index tuning, or a specific external format needs a new representation. Preserve interoperability instead of a wholesale universal-format rewrite.

## ADR-05 — Outcome-specific evidence and bounded scheduling

- **Context/requirements:** general-purpose learning must distinguish memory from reasoning and practical performance.
- **Alternatives:** one mastery percentage/flashcard engine for everything; separate subject engines; activity families with explicit outcomes/evidence and a recall scheduler adapter.
- **Evidence/trade-offs:** spacing/retrieval have substantial but moderated evidence; math/transfer/classroom findings are mixed, and a quiz cannot verify a real-world procedure. [Evidence register](../research/learning-science.md). FSRS reuse avoids writing an ad hoc memory algorithm; it does not validate practical skill estimates.
- **Decision:** four V1 activity families, immutable attempts and assessment methods, FSRS only for eligible recall, separate manual/fixed practice reminders, no universal mastery score.
- **Consequences:** honest progress is more nuanced; grading/self-rating/help are separate. New types use a versioned registry, not subject-specific tables or executable plugins.
- **Reconsider when:** suitable delayed/transfer evidence and reliable assessment justify an additional model; algorithm popularity alone is insufficient.

## ADR-06 — Preserve first, transform through reviewed stages

- **Context/requirements:** P02–P04/P12 require useful imports without AI, provenance, reimport and undo.
- **Alternatives:** replace sources with AI summaries/cards; universal lossy document conversion; preserve originals plus normalized/derived revisions and staged proposals.
- **Evidence/trade-offs:** extraction and pedagogical transformation have different failure modes. Preserving originals protects recoverability; larger asset storage and explicit fidelity reports are acceptable costs.
- **Decision:** typed handler pipeline, core text/Markdown/CSV/JSON/package formats, preserved attachments, conditional passive PDF processing, strict preview/validation/apply boundary. No implicit active content/network fetch.
- **Consequences:** unsupported formats can remain useful attachments; complex layout/OCR is deferred. Reimport uses stable keys/base revisions and never silently replaces annotations/history.
- **Reconsider when:** repeated real import needs justify another handler and its parser/security/license cost; never add a generic code execution escape hatch.

## ADR-07 — Optional scoped MCP through application services

- **Context/requirements:** P13/P14 require provider-independent agents without making AI necessary or granting DB/filesystem authority.
- **Alternatives:** GUI automation; raw database access; unrestricted localhost HTTP; stdio bridge to the application owner.
- **Evidence/trade-offs:** official MCP supports stdio; current 2026-07-28 protocol differs from legacy clients. SDK reuse reduces protocol work but requires pinned conformance and compatibility checks. [Versioning](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning), [stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio).
- **Decision:** opt-in stdio bridge/private scoped IPC, rich composable reads/authoring and persistent drafts, previews and validated proposals. Trusted UI creates exact approvals or bounded autonomy policies for reversible operations; sensitive changes require specific review. No remote HTTP or provider SDK in V1. See [agent contracts](mcp-contracts.md).
- **Consequences:** the application must be open, no hidden daemon. Host read consent is a disclosure boundary; a fully compromised OS account or an external host's later cloud use cannot be controlled by Memoquill.
- **Reconsider when:** a validated headless/remote workflow warrants an authenticated transport and its operational/security model, after separate threat analysis and authorization.

## ADR-08 — A pinned minimal glib upstream-fix backport

- **Context/requirements:** M1.1 reproduced an optimized `glib 0.18.5` string-iterator SIGSEGV under the pinned Rust 1.98.1 toolchain. GTK's actual `glib ^0.18` constraint prevents an interchangeable 0.20 upgrade; ignoring the defect is not a sound foundation.
- **Alternatives:** ignore the advisory or hide it by compiler pinning; force an incompatible API series; vendor an entire crate; select a reviewed immutable backport of the upstream correction.
- **Evidence/trade-offs:** [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), [upstream PR 1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343), the local failing optimized regression and passing patched regression. Comparison of every tracked backport file with the published crate found only the two mutable out-pointer changes in `src/variant_iter.rs`, plus an added patch README. This is a bounded compatibility fix, with an external-source maintenance cost.
- **Decision:** Cargo pins [the reviewed backport](https://github.com/andrew-wommack-ministries/glib-rustsec-2024-0429-backport/tree/7e91f1a9420fd3e6c58b7913bbbf201888304ea9) at `7e91f1a9420fd3e6c58b7913bbbf201888304ea9`. The normal check command runs an optimized iterator regression as well as debug tests. No API/version relabelling, optimizer workaround or broad advisory suppression.
- **Consequences:** build-source availability depends on that exact Git revision or its cached copy. Version-only scanners still report 0.18.5; bind the mitigation to the exact source/test evidence and retain unrelated/new advisory visibility. The dependency's original attribution remains separate from any future project-license choice; further licensing work is paused by the user.
- **Reconsider when:** the framework supports upstream fixed glib, another reviewed compatible correction becomes necessary, or source availability/maintenance fails. Remove the patch with an upstream upgrade only after the optimized regression and Fedora native gates pass. Do not turn a temporary backport into an unreviewed permanent fork.
