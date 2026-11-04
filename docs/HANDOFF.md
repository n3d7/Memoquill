# Memoquill architecture-first checkpoint

Updated 2026-11-04 14:58:43 UTC. Authorized phase: complete M2 and all substeps with local dev substep commits only.
User follow-up explicitly authorizes M2 application development after the final isolation correction. GUI/screenshots/push/merge remain prohibited.

- [Concrete contract](architecture/lesson-composition.md) and proposed ADR-09 now specify Unit-owned lessons, exact revisions, registry/actions/conditions, state/history, shared authoring, migrations/unknown types, budgets and four subject traces.
- Proposed lessonId = existing unitId, one active composition head per Unit. Different lesson tabs are independent and cannot alias mutable composition/draft/learner state. Linked-tab editing/synchronization and its future-view seam are excluded entirely. Existing resource/activity/block IDs and append-only personal history remain separate.
- Recommend a small owned renderer first; json-render remains an optional translating adapter. No new dependency, SQL, runtime API or feature was implemented.
- Independent builder check raised activity-occurrence identity and event/condition editing. Both were specified and reviewer confirmed resolution; no further question from that bounded check.
- KER-117 remains In Progress, description updated and read back; comment records local commit/review. Architecture agreement, not code completion, is the outstanding criterion.
- KER-118 owns replaceable UI/controller/adapters and local guest identity seam; real auth deferred. KER-119 persistence; KER-120 composer/renderer; KER-116 integrated qualification. These are not completed by the proposal.
- M3/KER-100 still owns actual book/file ingestion; KER-101 source portability. M2 source bindings do not imply parsers exist or authorize source uploads.
- M4/KER-102 owns real study/evaluation; M6/KER-105/106/107 scoped agent reads/authoring/security. Runtime preview cannot create real attempts. M8/KER-121 remains last, optional per lesson, default off and independent of core M7.
- Existing scientific screening and primary sources reused. [Evidence delta](research/lesson-contract-evidence.md) records narrow arXiv/Consensus gap checks and Firecrawl Cloud visibility documentation; no new learning-effect claim.
- User prefers native arXiv/Consensus for research and Firecrawl Cloud for web. Cloud credits checked before calls. No new service/configuration or private source upload.
- Graph Tier 2 checks are best effort, not a repository-wide completeness/security audit. Current RequestContext flag must not be described as an existing agent-grant system.
- Validation: 85 local links in nine architecture/docs files, whitespace and empty application source/test/config diff passed. No application tests or GUI qualification run during this hold.
- Branch dev began ea04fbd, 13 ahead of local origin/dev; architecture committed 3892fb9. Report/handoff are a separate local checkpoint. No push, PR, merge, remote CI or release performed.
- Preserve pre-existing modifications: .agents/skills/memoquill-task-workflow/SKILL.md, .gitignore, AGENTS.md; untracked reports 2026-11-01-143719Z-engineering-skill.md and 2026-11-01-155430Z-tool-catalog.md.
- Previous application commits cb1d0e6/b1cd960 and historical tests remain bounded evidence only, not proof of the constructor or current integrated acceptance.
- Physical GNOME/human audible Orca remains unresolved (KER-97 In Review); ADR-02 permits reversible development, not acceptance. Integrated M2/Fedora/release gates remain outstanding.
- Never capture screenshots or open GUI. Existing native-test workarounds are not authority to reconfigure host/security. Prior scanner evidence does not establish vulnerability freedom.

- User correction: lesson content/configuration/drafts/responses/assistance/reading preferences/schedules/progress stay isolated. Shared immutable definitions do not imply live updates or shared learner state. See [correction report](task-reports/2026-11-04-145843Z-ker117-tab-isolation.md).

Next: execute [M2 plan](superpowers/plans/2026-11-04-m2-composition.md) Task1 persistence, then frontend ports and composer, with per-task review/local commits. Preserve unverified physical/native acceptance; do not close M2 from unit tests alone.

Resume: [report](task-reports/2026-11-04-145327Z-ker117-lesson-contract.md), [requirements](planning/lesson-composition-requirements.md), [M2 contracts](planning/m2-contracts.md), [prior follow-up](task-reports/2026-11-01-201122Z-research-source-ingestion-followup.md), [scientific ledger](research/composable-learning-screening.md).
