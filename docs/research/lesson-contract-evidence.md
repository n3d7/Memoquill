# KER-117 contract evidence delta

Date: 2026-11-04. Bounded gap check, not a systematic review or efficacy trial.
Reused [architecture sources](composable-learning-architecture.md),
[scientific screening](composable-learning-screening.md) and the learning model.
Did not refetch prior catalog/H5P/WCAG/gamification/authoring papers.

## New retrieval and admission

- Native arXiv: `abs:"intelligent tutoring" AND abs:"authoring" AND abs:"interoperability"`,
  maximum 3, full abstracts, returned zero. This narrow result says nothing about
  literature absence; no additional arXiv claim was admitted or repeat search run.
- Consensus: `learning analytics interoperability assessment versioning context`,
  ten candidates returned. Selected fetch: Samuelsen, Chen and Wasson (2021),
  [Enriching context descriptions for enhanced LA scalability: a case study](https://consensus.app/papers/enriching-context-descriptions-for-enhanced-la-samuelsen-chen/a3ae355cf7df5b6984e67d4fad007fb2/).
  The search record describes an exploratory interview-based xAPI context study.
  Fetch record was retrieved; full publisher text was not examined. Retained as
  a future interoperability lead only, not proof of a particular Memoquill schema
  or learning effect. No xAPI/LRS dependency or new pedagogical requirement added.
- Firecrawl Cloud: balance before work 986 credits. One search, limit 2, for
  official json-render state/action/condition documentation; introduction/renderer
  excerpts returned. One targeted successful scrape of
  [Visibility](https://json-render.dev/docs/visibility), not the prior catalog page.
  Source documents state-path conditions, comparisons, compound conditions and
  repeat-scoped state; it notes renderer differences for filtered repeat lists.
  This supports evaluating an adapter as a translation boundary, not adopting a
  vendor storage format or treating visibility as authorization. Search reported
  2 credits used; accepted search feedback reported 1 refunded. Scrape's final
  billed total is not established by those figures; no additional paid jobs ran.

Contract decision (engineering judgment): use registry-declared typed field IDs,
strict bounded condition AST, no repeat/expression machinery, and service-owned
authorization. Recommend a small owned renderer first; json-render remains an
optional adapter if later compatibility/accessibility/resource evidence justifies it.
Neither library docs nor abstract-only research establishes educational efficacy.

## Current-source verification and limits

Codebase Memory Tier 2 project `home-void-Project-Memoquill`, ready at session start;
coverage generation `2026-11-04T14:50:43Z` reported no recorded issue for the checked
contract/source paths. This is best-effort coverage, not proof of completeness.
Graph lookup and exact snippet show `RequestContext` at
`src-tauri/src/workspace/types.rs:118` contains `trusted_ui: bool`.
An inbound CALLS trace returned zero for this struct; that does not establish absence
of usages or callers of its methods. No broad call-chain claim is made.
The proposal distinguishes future actor/grant context from this present flag.

The existing logical data model was read to verify Unit/resource/activity/history
separation and shared-object authorization in MCP contracts. A local-fast extraction
of the data-model document was provisional; direct source reading supplies evidence.
This checkpoint does not audit the complete implementation, migrate storage, run
application tests or qualify native accessibility.

Deliverable: [concrete architecture proposal](../architecture/lesson-composition.md).
User agreement and subsequent implementation planning remain outstanding.
