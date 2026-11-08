# Post-M2: isolated course workspaces

User-confirmed requirements, 2026-11-08. Separate step after M2 integration;
tracked as [KER-127](https://linear.app/kerigan/issue/KER-127/post-m2-establish-isolated-course-workspaces),
Backlog. This is a requirement/scope record, not an approved implementation design.
M2 PR #3 is merged; all its original commits remain reachable from main.

## Course boundary

The top-level sidebar selects courses, for example Quantum Physics, Mathematics
and AI/ML. Each tab/workspace is a whole course with its lessons and tasks inside.
It owns its materials/source bindings, lesson structure, UI configuration,
drafts, responses, history, search scope and agent context. Course switching and
asynchronous work must not mix these states or disclose another course's data.
No linked mutable views or future linked-tab seam is requested.

This changes the M2 lesson-tab boundary. A design must explicitly map existing
Subjects, Units/lessons, shared placements and resources into course ownership;
renaming existing labels is not sufficient. Preserve user data and source/history
provenance; migration and recovery behavior remain to be designed.

## Learning and agent direction

The learner defines what to study, the goal/focus and subsequent changes in
direction. External AI through provider-independent MCP may progressively gather
sources, compose the course and adapt the learning path using observed outcomes.
Books, research and user-provided course files/folders (such as docs) are inputs;
their authorized read scope and actual processed coverage must remain explicit.
No fixed provider/model/tariff or AI-budget answer is an application prerequisite.

The learner can select suitable practice formats. Deep changes to a course's
content, structure and presentation are desired; full visual design stays deferred.
Effectiveness claims require applicable evidence; autonomy/AI grading/assistance
policies are still provisional. The [evidence brief](../research/ai-directed-learning-brief.md)
distinguishes existing contracts, new research and unresolved design choices.
This step does not implement arbitrary executable UI or agent edits to past grades.

## Separate design work before code

1. Agree course identity, ownership and upgrade behavior for existing shared data.
2. Define course-scoped services/DTOs/controllers and nested lesson navigation;
   retain offline core behavior and portable boundaries, targeting X11 only now.
3. Specify switch/reopen, pending async work, drafts, recovery and scoped AI grants.
4. Define proportionate two-course isolation and migration/restart acceptance.
5. Align later ingestion/study/MCP plans after agreement. No new milestone/blocker
   or M3 execution is inferred from this record.

Next: continue the requirements discussion, then compare approaches and approve
a design before an implementation plan. Do not push this planning checkpoint
into the completed M2 PR or begin the course migration from its Backlog status.
