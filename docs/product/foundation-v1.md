# Foundation V1 scope

## Interface language

Foundation V1 ships Memoquill's interface and application-authored messages in **English only**. Do not add other UI locales or a language selector in this release. Learning materials and user input remain subject-agnostic and Unicode-capable; multilingual technical input tests are not interface translations or additional example curricula. Future interface localization is outside the current release scope.

## Product promise

Memoquill helps an individual turn their own information into organized study and practice, return to it over time, and preserve the evidence of what they have attempted. It supports arbitrary subjects and distinguishes remembering from understanding, solving, and performing. English and Python are initial examples. Mathematics/quantum physics and cooking are mandatory architecture/test scenarios.

V1 is a Linux desktop product, particularly Fedora. It is useful offline with no AI, account, cloud subscription, or server. It is a coherent personal learning environment, not a complete curriculum generator or a guarantee of learning effectiveness.

## Prioritization rule

**Essential:** required for a complete create/import → understand/practice → feedback/review → progress → recovery loop, or to protect its data/security/accessibility. **Conditional:** significant value but a dependency/feasibility gate must justify its release cost. **Post-V1:** no necessary role in that loop or disproportionate complexity. Conditional capabilities cannot silently become release prerequisites; a failed gate needs a documented fallback and journey review.

| Capability | Classification and bounded release behavior |
| --- | --- |
| Custom subjects, goals, arbitrary-depth collections, tags, links, saved learning paths | Essential; no subject templates required. Tree navigation plus typed relationships; no graph database/canvas. |
| Source library and manual authoring | Essential; retain originals, create rich text/code/math/table lessons, cite sources, navigate/search, and resume reading. |
| Text/Markdown, mapped CSV/JSON, Memoquill package imports | Essential; preview, validate, deduplicate, partial recovery, provenance, and reversible application. |
| PDF files | Original preservation and explicit opening are essential. Born-digital text extraction and in-app passive viewing are conditional on parser/packaging/security qualification in M3. Attachments remain usable with clear extraction status if that gate fails. |
| Images/diagrams and media | Essential as file attachments, captions/alt text, and explicit viewing. Optional audio/video playback where the packaged runtime supports it; no bundled pronunciation service. |
| Activities | Essential: recall/cloze, single-choice, constructed response/problem and procedure/checklist. Shared presets/phases cover explanation, optional prepared teach-back, worked-example completion/fading, error repair and changed-context application. Matching/ordering remains conditional; no executable plugins or native drawing engine required. See the [evidence portfolio](../research/learning-activities.md). |
| Study/practice/review sessions | Essential: explanation + example + attempt, optional hints, suitable feedback, user pacing, durable interruption/resumption, due recall and manually planned practice. |
| Topic/subject assessment | Essential as a user-selected, frozen session over objectives/activities with delayed solution reveal when selected. No psychometric exam engine. |
| Scheduling | Essential for eligible recall using FSRS behind a narrow adapter; user workload controls. Fixed/manual practice reminders for other outcomes. Optimization/personal model fitting conditional after enough valid history; no V1 requirement. |
| Progress | Essential: attempts, evidence source, per-outcome history, uncertainty/unassessed states. No universal mastery percentage. |
| Personalization/accessibility | Essential: light/dark/system, text sizing, keyboard shortcuts, accessible focus/navigation, session preferences, saved views, per-subject overrides. Custom theme import conditional on contrast validation. |
| Ownership/recovery | Essential: versioned portable export, verified full backup/restore including assets and history, migration safety, archive/undo. |
| External AI/MCP | Essential: opt-in local stdio, rich scoped discovery/read/authoring, resumable drafts/jobs, validated previews/proposals, UI-owned review or bounded autonomous reversible application, audit/revocation. No direct DB access, permission self-grant or history/grade mutation. Implementation follows stable services in M6. |
| Advanced import/analysis | OCR, EPUB, remote website crawling, complex HTML fidelity, automatic pedagogical extraction and semantic duplicate detection are Post-V1. Handlers have a documented extension contract. |
| Programming execution | Post-V1. V1 supports code reading, writing, debugging explanations, expected outputs, and rubric/self-assessment; imports never execute. |
| Platforms/infrastructure | Windows, macOS, mobile, sync, collaboration, accounts, marketplace, mandatory daemons, hosted AI, built-in model-driven agents, and complex visual programming are Post-V1. External-agent orchestration under explicit MCP authority is included. |

PDF-heavy journey C remains a valid conditional scenario: every PDF original is retained and can be explicitly opened or read by a scoped agent; automated local extraction is not promised when unsupported. The release must disclose this limitation before import, never pretend that an attachment was parsed.

## Release demonstration

A fresh offline installation must create a subject, import a licensed collection, view sources, create activities, complete/resume sessions, receive feedback, review eligible items, explain progress, customize the workflow, export, and restore. M7 repeats this across all four subject types. Only English and Python need small user-facing example packs; neither is embedded in the schema or navigation.

Initial packs should be original, reviewed material with an explicit data license and per-source attribution. A curated English CSV and Python concepts/code-reading pack are sufficient; do not redistribute Oxford 5000 or third-party audio. [Dataset evaluation](../architecture/third-party-reuse.md) distinguishes a legally available lexical source from a complete language curriculum.

Release blockers include lost history, unverifiable recovery, unauthorized agent access/application, imported execution, inaccessible core keyboard/screen-reader workflows, and any hardcoded subject exception. Targets and evidence are defined in [verification](../quality/verification-strategy.md); no release readiness is claimed now.
