# Product specification

## Users, context, and boundaries

Primary users are independent learners organizing mixed personal material on a Linux desktop, including people with no AI experience and people using keyboard/assistive technology. Novices need an obvious next action; experienced users need search, efficient bulk organization, and control over study. A local workspace has one learner in V1, with no registration. Multiple workspaces may isolate data; multiple learner/account infrastructure is deferred.

Goals: reduce content-entry/organization overhead, support appropriate practice, make returning and recovery reliable, and keep user control. Non-goals: guarantee expertise, replace teachers or real-world assessment, certify competence from quizzes, run arbitrary programs, or generate a universal curriculum automatically.

## Functional acceptance requirements

| ID | Observable requirement | Acceptance evidence |
| --- | --- | --- |
| P01 | Create any named subject, optional prior-knowledge note/goals, arbitrary-depth organization, and links without a template. | Quantum physics created from blank; reorder/move while identities, references, and attempts survive. |
| P02 | Separate sources/material from activities, reuse a resource in several locations, and keep attributable originals. | One source appears in two collections without copying its bytes; original hash verifies after edit/reimport/export. |
| P03 | Author explanations with code, Unicode, tables, math, and attachments; reading remains useful without activities. | Four-subject content round trip; unsupported imported structures retain source and a visible warning. |
| P04 | Preview mapped imports and report accepted, rejected, duplicate, unchanged, and unsupported items. | CSV/JSON field correction and interrupted-job retry create no silent duplicates or partial live structure. |
| P05 | Create/import each V1 activity using ordinary forms and source links. | Authoring needs no AI or schema editing; malformed payloads rejected with actionable field errors. |
| P06 | Start, answer, reveal feedback, pause, and resume a session with exactly-once submitted attempts. | Kill/restart preserves acknowledged attempts and recoverable drafts; a repeated submission does not count twice. |
| P07 | Explain each activity's assessment mode and separate response, correctness, self-rating, and observed/declared/unknown help. | Self-rated cooking never displays verified practical mastery; a choice result records its cues/target reasoning without implying uncued production. Unknown assistance is never silently labeled unaided. |
| P08 | Offer due recall and planned practice, sensible workload limits, and transparent schedule controls. | Change daily budget without deleting attempts; overdue items remain available; clock/DST tests pass. |
| P09 | Show meaningful per-objective/outcome history, recent mistakes, coverage, and unassessed/stale evidence. | No aggregate substitutes self-report for independent correctness; users can inspect contributing attempts. |
| P10 | Search and navigate sources, units, activities, tags, and text; restore reading/session context. | Search finds multilingual/code examples and opens the source anchor; uncertain/missing anchors are flagged. |
| P11 | Support personalization with accessible defaults and reset/recovery. | Appearance, keyboard, saved view, and subject preference changes preserve core actions/data. |
| P12 | Export and restore content, sources, preferences, progress, and attribution independently of a service. | G1 native full recovery includes linked-source snapshots; G2 portable reimport preserves its declared subset. Both detect corruption before activation and disclose omissions. |
| P13 | Authorized external agents can inspect and author rich learning structures through composable typed tools, resumable drafts, previews/proposals and bounded delegated autonomy. | Default review and explicitly delegated reversible application both work; revoked/out-of-scope/sensitive requests fail or request review. Every apply is validated/transactional; no agent grants itself authority or writes learner history/grades. |
| P14 | Core actions run offline; network and AI are optional boundaries. | End-to-end journey with network disabled and no configured AI; imported remote references cause no background requests. |

## Non-functional requirements

- **Integrity:** atomic content/proposal/import application; durable acknowledged attempts; explicit migrations, backups, repair reports, and source/history revision links. Recoverable failure is preferred over optimistic silent repair.
- **Accessibility:** applicable WCAG 2.2 A/AA expectations interpreted for desktop through WCAG2ICT; keyboard and Orca/AT-SPI qualification of actual packaged core workflows. No conformance claim before evaluation. See [UX evidence](../research/educational-ux.md).
- **Performance:** evaluate on a documented supported Fedora baseline with 100 subjects, 10,000 resources, 100,000 activities, 1,000,000 attempts, and 10 GB of file assets. Provisional goals: warm launch to useful view ≤3 s; common search p95 ≤500 ms; no synchronous parsing/indexing that blocks UI interaction for >100 ms. Measure hardware, memory, cold/warm state and exceptions; these are budgets, not current results.
- **Resource use:** bounded imports, queries, MCP payloads, and workers; cancel/report progress. Large collections use pagination/virtualization without breaking accessibility. No vector database, GPU, or model runtime required.
- **Portability/maintenance:** Linux-only delivery with platform adapters at filesystem/IPC/packaging boundaries; versioned domain formats and typed application services. No speculative sync protocol or platform implementations.
- **Privacy:** no telemetry or cloud upload by default; local diagnostics redact responses/source text and credentials. Exports disclose whether personal progress is included. Reading by an external agent is explicitly treated as disclosure to that agent's host.
- **Localization:** locale-aware display and message catalogs from the first UI; source content languages independent of interface language. Initial English UI, with Russian translation conditional on reviewed coverage; no hardcoded learner-language schema. Test RTL/complex Unicode content even if full RTL UI is deferred.

## Default information architecture

A restrained shell exposes Library, Learn, and Progress; settings and agent connections are secondary. The start view offers Resume, Due review, and a visible Create/import action. A subject workspace combines a collection outline, searchable material/activity list, and a reading/practice pane. Stable text labels, breadcrumbs/back, visible save state, and a command/search entry support efficient desktop use. This layout is a design hypothesis requiring task evaluation, not a scientific optimum.

## Ownership guarantees

The learner can access originals, inspect provenance/revisions, export without a service, back up full history, and choose which content/progress agents can read. Reorganization, reimport, editing, and archive never silently delete attempts. Human/imported/AI authorship and review state stay visible. Version changes that invalidate earlier evidence prompt an explicit decision; historical attempts retain their original question/rubric.

Requirements map to [journeys](user-journeys.md), [roadmap](../planning/roadmap.md), and [verification gates](../quality/verification-strategy.md). Remaining high-impact uncertainties are the Fedora runtime/accessibility gate, PDF processing feasibility, usability of a flexible organization model, and real-world scheduling applicability beyond recall.
