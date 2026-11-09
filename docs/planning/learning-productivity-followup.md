# Course autonomy and learning-productivity requirements

Execution amendment, 2026-11-08: separate post-M2 implementation is authorized
by the user's continuation request; the earlier planning-only wording below is
historical. [Course design](../superpowers/specs/2026-11-08-course-workspaces-design.md)
settles ownership/migration, and [extension contracts](../architecture/course-extensions.md)
define the data-only future-tool foundation. Other main-stage assignments and
X11-only delivery remain unchanged.

Consolidated user decisions, 2026-11-08. Planning complete; implementation not
started. M2 PR #3 is merged. This work is a separate local post-M2 checkpoint.
The main M1–M8 delivery target remains Linux/X11 with portable contracts.

## Confirmed intent

- Optimize useful learning per time/effort: durable independent recall,
  explanation, problem solving, transfer or practical performance as appropriate.
  No universal maximum or guaranteed gain follows from this product goal.
- The learner sets what to study, goals and focus, and can revise direction.
  AI should take as much useful planning/adaptation/checking work as justified.
- Any capable external model may operate through MCP; no provider/model/tariff
  or budget answer is a product prerequisite. Capability/reliability still varies.
- A sidebar entry/tab is a whole isolated course, with lessons/tasks inside.
  Content, sources, UI, forms, responses, history, search and AI context stay scoped.
- AI may progressively assemble a course using authorized research/tools, books
  and supplied files/folders such as course docs. Preserve provenance, actual used
  ranges and partial/unread coverage; source intake and synthesis are distinct.
- The learner can choose suitable practice: tests, tasks, cards, repeats or other
  supported formats. Applicability evidence informs pedagogy; neither one format
  nor unlimited repetition is automatically optimal or mandatory.
- Courses can deeply change in content/presentation. Provide a foundation for
  future AI-created interactive tools beyond built-in types; actual simulator
  implementations and an execution runtime are not approved by this requirement.
- Before changing learner-edited material/tasks/UI, AI asks what is permitted
  and waits for an answer. Apply only within that authority; preserve versions.
- Offer opt-in per-course research schedules, e.g. weekly. Return justified,
  relevant, non-duplicate proposals; no useful update is a valid result. Applying
  updates has separate authority, including protection of manual edits.
- Abruptly interrupted agent work waits for fresh permission to continue,
  despite prior permission. A checkpoint does not authorize automatic replay.
- Full visual design remains deferred. Questions are for material uncertainty
  about user intent/scope; choose pedagogical details from applicable evidence.

## Executable backlog mapping

All new children remain Backlog. Creation is not implementation authority.

| Stage / parent | Substep or existing acceptance | Scope |
| --- | --- | --- |
| Separate post-M2 / KER-127 | [KER-128](https://linear.app/kerigan/issue/KER-128) | Course identity, ownership, existing-data migration/recovery |
| Separate post-M2 / KER-127 | [KER-129](https://linear.app/kerigan/issue/KER-129) | Functional sidebar/nested lessons, state/service/AI-scope isolation |
| Separate post-M2 / KER-127 | [KER-130](https://linear.app/kerigan/issue/KER-130) | Versioned interactive-extension foundation, capabilities/inert fallback |
| M3 / KER-100 | Existing intake acceptance amended | Course sources/books/folders, bindings, provenance and coverage |
| M4 / KER-102 | [KER-131](https://linear.app/kerigan/issue/KER-131) | Immutable observed attempts versus self/deterministic/AI assessment judgments |
| M4 / KER-103 | Existing scheduler/assessment acceptance amended | Recall eligibility, independent outcomes, versioned adaptive-policy evidence |
| M5 / KER-104 | Existing progress/preferences acceptance amended | Learner goals/focus/format choices, uncertainty and inspectable outcomes |
| M6 / KER-106 | [KER-132](https://linear.app/kerigan/issue/KER-132) | Progressive goal-led authoring, feedback and course adaptation through MCP |
| M6 / KER-106 | [KER-133](https://linear.app/kerigan/issue/KER-133) | Course research schedules, relevance and justified update proposals |
| M6 / KER-107 | Existing autonomy acceptance amended | Ask-before-changing manual work, scoped consent, interrupted-job permission |
| M7 / KER-108 | Existing release qualification amended | Supported-X11 isolation/recovery/permission and honest evidence presentation |

Course-boundary contracts must be settled before dependent course-scoped
implementation. Existing M3/M4/M5/M6 prerequisite order remains; only the listed
parent/child relations were added, not extra blocker edges or new milestones.
Other-platform work stays separate/inactive after M8.

## Evidence and explicitly chosen defaults

Reuse [learning science](../research/learning-science.md),
[activity evidence](../research/learning-activities.md) and the
[AI-directed brief](../research/ai-directed-learning-brief.md). Distinguish empirical
findings, context-dependent applicability, engineering choices and hypotheses.
AI judgment/confidence, assisted correctness and task counts are not independent
competence. Evaluate assessment error and learning benefit separately; use delayed
or changed-context evidence where the claim needs it. M4 stores the evidence;
live external-agent workflows belong to M6, with no mandatory built-in provider.

Conservative operational default selected here: a missed unstarted research run
is marked due/missed and awaits explicit run/catch-up permission on reconnection.
An interrupted run always awaits fresh continue permission (user requirement).
Regular opted-in scheduled runs are distinct from catch-up/resume. The missed-run
default is not claimed as a research-proven learning optimum. No host daemon,
external schedule, provider connection or actual recurring task has been started.

Next: design the separate course boundary/migration and extensibility contracts
using the existing evidence; ask only for material intent/scope uncertainty.
