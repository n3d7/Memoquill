# Architecture research, roadmap and learning-evidence guidance

Completed the requested research/tracker/documentation checkpoint. Product feature/UI development and further GUI qualification remain on hold by explicit user instruction. No revised constructor, account service or gamification runtime was implemented.

User requirements now cover manually composable lesson experiences and equivalent later MCP authoring; replaceable frontend screens/controllers/service adapters; applicable scientific learning evidence; optional per-lesson gamification as the final phase; and a guest-first identity seam for future registration/login.

Created KER-117–120 under M2 and made KER-116 depend on them. KER-117 is In Progress for architecture agreement; the other new tasks are Backlog. Updated KER-99/M2, KER-109/113/115/116, KER-102/104, KER-105/106/107 and KER-108. Added the final M8 milestone and KER-121 (Backlog), following KER-108. Persistent reads verified new issue parents/statuses/milestones, M8 last ordering and default-off/per-lesson requirements. Existing Done slices were not reopened or presented as new-architecture acceptance.

Synchronized `technical-design.md`, `data-model.md`, product customization, roadmap and M2 contracts. Added [requirements/task mapping](../planning/lesson-composition-requirements.md) and [bounded research](../research/composable-learning-architecture.md). Aggregate ownership, precise schemas/interaction semantics, resource budgets and renderer dependency selection remain open design decisions.

Studied json-render catalogs/registries, H5P learning-component semantics, MCP security boundaries, meaningful reading order and two publisher-verified gamification reviews found through Consensus. These support architecture patterns and conditional evidence, not a guarantee for every learning method or reward mechanic. Public raw retrievals stay ignored under `.qualification/m2/adaptive-ui-research`; duplicate registry retrieval was not counted as independent evidence. Local-AI extraction was provisional and material facts were checked against source.

Created local project skill `.agents/skills/memoquill-learning-evidence/SKILL.md` with evidence workflow, installed research-skill links and scientific/web/technical MCP routes including Consensus, arXiv and Firecrawl selfhost/Cloud. Linked from local AGENTS. Skill validator, YAML/local-link checks and repository-aware catalog check passed. No claim of behavioral prompt-evaluation or a learning-effectiveness study. Personal ignored skill/guide files were not force-added to Git.

Validation: changed-document links/whitespace and tracker readback; application source/test/config diff remained empty after the hold. No application tests/GUI actions were continued during the architecture phase. Prior source commits are `cb1d0e6` and `b1cd960`; no push, PR, main merge, release or acceptance closure is claimed.

Used: engineering-workflow/Superpowers brainstorming as the architecture phase owner; skill-creator, writing-for-agents and Memoquill workflow/context/reporting/routing. MCP/plugin calls: Codebase Memory, local_ai, Consensus and Linear. Firecrawl plugin instructions were read; actual public retrieval used the existing selfhost CLI (initial restricted SSH probe failed, same host-execution route succeeded without configuration changes). Cloud/other research tools were inventoried, not executed. RTK, shell/Git, Python, YAML/skill validator and apply_patch were used. No screenshots were captured.

Next: a concrete KER-117 design and its agreement before an implementation plan or product code resumes. Existing offline, source/progress integrity, security, GNOME/Orca and publication gates remain intact.
