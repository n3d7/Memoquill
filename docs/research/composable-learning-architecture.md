# Composable learning: architecture and evidence

Status: bounded research and requirements clarification, 2026-11-01. Product feature/UI development and GUI qualification are on hold until the revised architecture is agreed. No new runtime dependency, MCP server, account service or product code was introduced by this research.

## User requirements

- People compose their own lessons and learning methods from content, exercises, tests and interaction/presentation blocks. Manual authoring is first-class and works offline.
- Each lesson tab owns its composition and local configuration. An agent can perform the same authoring operations through MCP; changing the global application shell is a separate concern.
- Developers can replace screens, component implementations and the design system through narrow interfaces, without rewriting storage or domain rules.
- Educational decisions and Memoquill-specific agent skills use applicable research, clear objectives and proportionate learning checks. Custom experimental methods remain possible and are not mislabeled as proven.
- Optional gamification is a separate final phase, enabled per lesson rather than globally. Future registration/login needs a seam now, not a mandatory account or a pretend login flow.

## Current evidence and gaps

The existing data model already distinguishes units/paths, reusable resource revisions, activity revisions and personal learning records. The technical design already requires UI and MCP to use the same application services. Current M2 implementation supplies organization, canonical documents, authoring and native services, but its fixed forms/editor are not the requested composable lesson constructor. The old M2 contract freeze does not settle composition ownership, registries, interaction schemas or view replacement.

Keep those existing identities and content codecs. First resolve whether a user-facing lesson is a projection of the current unit/resource concepts or requires a distinct aggregate; do not invent duplicate lesson/content/history entities through UI terminology alone.

## Primary sources and applicability

| Source | What it supports | Limit |
| --- | --- | --- |
| [json-render catalog](https://json-render.dev/docs/catalog) and [registry](https://json-render.dev/docs/registry) | A bounded vocabulary/schema can be separate from platform component implementations and named action handlers. | Engineering precedent, not evidence that a lesson teaches effectively or that the library is a security boundary. |
| [H5P semantics](https://h5p.org/semantics) | Typed configurable learning components can expose machine-readable parameters to an authoring editor. | Not a decision to embed H5P, its runtime, dependencies or content libraries. |
| [MCP security guidance](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices) | Preserve explicit authority and trust boundaries; avoid a confused deputy and token passthrough. | OAuth/HTTP-specific mechanisms must not be copied blindly into the planned local stdio bridge. App-level scopes remain necessary. |
| [WCAG meaningful sequence](https://www.w3.org/WAI/WCAG22/Understanding/meaningful-sequence.html) | Recomposition must preserve a programmatically determinable meaningful reading order. | This standard does not establish learning efficacy; keyboard/focus behavior also needs native qualification. |
| [Sailer and Homner, gamification meta-analysis](https://link.springer.com/article/10.1007/s10648-019-09498-w) | Aggregate positive cognitive/motivational/behavioral effects, with heterogeneity and less-stable motivational/behavioral estimates in methodological subsplits. | Does not validate every badge, level or scoring rule, or guarantee transfer/long-term benefit for Memoquill. |
| [Gamification, intrinsic motivation and psychological needs](https://link.springer.com/article/10.1007/s11423-023-10337-7) | A review/meta-analysis reports context-dependent motivational effects and challenges around competence/autonomy. | Motivational constructs are not interchangeable with demonstrated competence or durable learning. |

Public sources were searched/retrieved through existing Firecrawl selfhost; Consensus supplied gamification paper candidates, then publisher sources were inspected. The `/docs/actions` retrieval duplicated the registry page and is not counted as independent evidence. Local-model extraction was provisional; consequential source claims were checked directly. This was not a systematic review or a Memoquill effectiveness study.

## Recommended architecture to specify next

1. **Separate four kinds of data.** Canonical content and provenance; activity definitions/evaluation policy; lesson composition/presentation and component references; personal attempts/history/progress. Use stable IDs, explicit versions and reference/migration rules. Presentation edits must preserve content anchors and learning history.
2. **Own a versioned contract.** Store bounded, validated declarative data rather than generated React/HTML/JavaScript. A registry describes supported component types, prop schemas, slots, events/actions, state boundaries, accessibility and migration/fallback behavior. Components can evolve without making a vendor schema the sole source of truth.
3. **Separate presentation from orchestration.** Presentational components receive typed view models/actions. Controllers own interaction/retry/state behavior. A frontend application port has native and test adapters; only the native adapter speaks Tauri IPC. Rust application services retain validation, authority, transactions, receipts and repositories. A screen replacement should pass the same contract scenarios without a storage migration.
4. **One authoring path.** Manual controls and later MCP adapters call the same semantic operations. Validate proposals, preview their exact diff, then apply through current revisions/idempotency and explicit review or matching delegated policy. MCP can edit only authorized lesson objects, not arbitrary React code, database paths, global shell or personal outcomes.
5. **A real manual composer first.** Add/edit/remove/reorder/nest supported blocks, configure activities and choose presentation inside a lesson tab. Provide undo, keyboard alternatives, clear unsupported-type recovery and a preview using the same renderer as the lesson. Keep author preview outcomes separate from real study results.
6. **Safe interaction semantics.** Named trusted actions and bounded typed conditions/state transitions; no arbitrary scripts, dynamic imports, unrestricted CSS/selectors, eval, filesystem or network actions in a lesson payload. Validate reference scope, containment/cycles, bytes/nodes/depth/work limits, state paths and action capabilities. New executable component implementations require an independently reviewed extension path.
7. **Future identity seam.** Default to a local guest actor behind a small identity/session port. Keep account identity separate from MCP grants and workspace authority. Document future profile/account linking and data ownership; defer provider/protocol, password handling and actual registration until their own design/security work. No login requirement for core offline use and no nonfunctional registration UI.

The recommended direction is a project-owned contract plus a trusted renderer registry. Compare a small owned renderer with an adapter around json-render in a bounded spike before choosing a dependency. A form-only builder is insufficient for arbitrary learning composition; arbitrary generated executable UI adds an unbounded code-execution surface. No renderer dependency is selected by this document.

## Optional per-lesson gamification

Plan this after the core phases as M8. Each lesson can opt into user-defined achievements, levels, scores/ratings, percentages and scales/progress bars through versioned rules over scoped learning events. Explain denominators, thresholds and evidence provenance. Distinguish completion, game score, self-report and independent performance; a reward percentage is not universal mastery.

Default off; allow per-lesson enable/disable/reset with explicit history semantics. No global XP/account level, coercive default streak or cross-lesson effects. Custom safe rules remain available as experiments. Recommended presets need applicable evidence and a proportionate benefit/cost check using target learning outcomes, learner time, autonomy and accessibility. No paper guarantees that every configurable mechanic works. Keep rule execution bounded, auditable and separate from assessment evidence; expose parity through the same future authoring services/MCP scopes.

## Architecture acceptance before feature work resumes

- Resolve aggregate ownership, reference/version/migration contracts, component/action capability vocabulary, frontend ports and state/history separation.
- Trace each user requirement and stage boundary to Linear; define a minimal first manual composition/preview vertical slice. M4 still owns durable study behavior/evaluation, M6 the live agent adapter, and M8 optional gamification.
- Include manual/agent command equivalence, invalid/deep/cyclic payloads, unauthorized cross-lesson refs, unknown-node preservation, exact retry/conflict behavior, revision/anchor/history continuity, offline save/reopen, keyboard/focus/reading order and adapter replacement tests.
- Reassess the existing native acceptance matrix after architectural agreement; previous screenshots establish the initial fixed UI only. No current code or local test result proves the new constructor exists.
