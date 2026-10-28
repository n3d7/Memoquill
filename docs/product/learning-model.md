# Subject-agnostic learning model

## Organization and meaning

A **subject** is a user-named scope with goals/preferences. Its **collections** are an arbitrary-depth containment tree with user labels (module/topic/chapter are conventions, not tables). Entries reference shared resources and learning units; a resource or unit can appear in multiple collections and subjects without losing identity. Tags, search, and saved views provide alternative access.

A **resource** is learning material, original asset, or authored explanation with immutable revisions and provenance. A **learning unit** expresses a manageable objective and links useful resources, activities, and prerequisites. Units can be created gradually; importing files need not invent units automatically. A **learning path** orders unit references and optional stages. Prerequisite edges between units/objectives form a DAG; related/reference links may cycle. Prerequisites are advisory unless a user explicitly selects a gated path. No graph editor or graph database is needed in V1.

This hybrid permits familiar outlines and shared concepts without imposing `Subject → Module → Topic → Unit` as a rigid ontology. Moving a collection entry changes location, not the identity/revision of its content or the learner's history. A subject can consist initially of one resource and one activity.

## Outcomes and evidence

Objectives describe an action and context, optionally with criteria. Outcome facets include recall, explanation/relationships, problem solving, language production, and procedural performance; these are extensible labels, not exclusive subject types. Evidence records what happened, how it was assessed, whether hints/solutions were used, and what remains unknown.

| Activity family | Response and feedback | Valid inference and limit |
| --- | --- | --- |
| Recall / bounded cloze | Attempt before reveal, compare with reference, learner rates recall; optional exact-answer check clearly labeled. | Recall of this prompt under these conditions. FSRS eligibility requires repeatable recall; not general understanding. |
| Single-choice | One selected option; authored correct key, explanation, and distractor feedback. Questions can require recall, discrimination or inference. | Successful selection with these cues. Neither universally inferior to open recall nor evidence of uncued production; guessing/cueing and item quality matter. |
| Constructed response / problem | Written explanation, derivation, code, free recall, or debugging; optional staged hints, worked solution, and rubric. | Persist response and self/rubric assessment separately. No semantic auto-grader, symbolic equivalence engine, or code execution promised. |
| Procedure / practical task | Ordered steps/checklist, observations, optional artifacts, reflection and rubric. | Self-reported completion/observations, not independently verified cooking or motor competence. |

Worked examples/explanations are resources; a solve-and-compare activity links them. A passive read is not recorded as a correct retrieval. A question may target multiple outcomes, but its evidence remains assessment-specific. The activity type registry owns schemas/renderers/evaluation policies and migrations; adding a declarative type does not require redesigning every relational table. Executable plugins are deferred.

### Useful variants and instructional phases

The [activity portfolio and additional evidence](../research/learning-activities.md) refine these families without creating separate engines for every technique. V1 supports focused self-explanation, optional prepare-and-teach-back, completion/fading, clearly marked error diagnosis/correction, and changed-context application through ordinary resources and constructed responses. Matching/ordering and native drawing/graph tools remain conditional/post-V1 as documented there.

A session can link bounded **attempt → instruction/example → completion → independent application → feedback/revisit** phases, or begin with instruction when appropriate. Not every sequence needs every phase. Productive-failure evidence supports designed attempt-first sequences with subsequent consolidation; it does not justify unbounded unguided failure. Phase, response format, target outcome, reference/rubric and assistance state are explicit and versioned. Deliberately erroneous instructional examples must be labeled and have a correct resolution.

Record app-observed hints/reveals separately from learner-declared external help and unknown assistance. A response with unknown external assistance is not silently classified as unaided. Learning/practice may expose examples and feedback; an independent check uses declared aids, frozen items and an explicit feedback policy. Optional AI help policy and MCP authoring authority are different concerns.

**Assessment** is a session mode selecting activities by subject/collection/objective, freezing their revisions and feedback policy. Topic and subject assessments use the same engine. Results show coverage and outcome-specific evidence with unassessed areas, not an invented universal mastery number.

## Learning-session policy

The default loop is goal/context → concise explanation or worked example if needed → attempt → feedback/compare → next step. It is learner-paced, with the designed attempt-first option above. Returning users can resume or review due recall in one action; new users see create/import and a short example rather than a complex dashboard.

Apply spacing/retrieval to eligible memory tasks; allow related mixed practice after initial examples/blocks. Interleaving is a selectable plan policy, not indiscriminate shuffling across unrelated subjects. Reduce scaffolding as competence evidence accumulates or the learner requests it; never remove needed accessibility support. See the [evidence register](../research/learning-science.md) for the limits of these product inferences.

Use FSRS for recall scheduling with explicit algorithm/parameter version and immutable review logs. Store rating, elapsed time, help and response outcome separately. Other activities can be repeated through fixed/user-planned reminders without a false memory-stability estimate. Workload settings change selection/due handling, not historical records. Sparse history uses documented defaults; personalized optimization is conditional and reversible.

## Four contrasting architecture cases

| Subject | Material and structure | Practice and evidence | Common mechanism |
| --- | --- | --- | --- |
| English | Licensed lexical CSV, sense/context, authored sentences, optional separately licensed audio; vocabulary is only one part of a path. | Recall a meaning; produce a new sentence with rubric; record mistakes. Definition recall never proves conversational proficiency. | Shared resources, units, activity families, source attribution, and recall schedule. |
| Python | Concepts → functions → mutability; source snippets/docs and prerequisite links. | Trace output, explain a bug, write non-executed code, compare worked solution, reflect against rubric. | Constructed response/code blocks and ordinary prerequisite edges; no `programming_subject` entity. |
| Quantum physics | Complex numbers/vector prerequisites, a two-state-system explanation with LaTeX, derivation and related concepts. | Solve a normalization/probability problem, expose hints, compare a worked derivation; assess reasoning separately from memorized notation. | Math document blocks and constructed responses; prerequisite DAG, no physics-specific schema. |
| Cooking | A complex recipe links knife technique, emulsification, timing, and food-safety reference resources. | Recall a temperature fact separately from completing ordered actions and recording observations. | Procedure steps, resources, recall, reflection, and practice reminders; no quiz-derived competence certification. |

No real-world safety-critical instruction is validated merely by entering it. Example content needs subject review; the platform records provenance and uncertainty rather than supplying scientific authority to imported/AI text.

## Content changes

Attempts bind to immutable activity/solution/rubric revisions. Fixing a typo preserves identity/history; changing what a question measures flags existing evidence as potentially stale. A deliberate replacement creates a new activity or explicitly starts a new schedule generation, retaining prior attempts. Reimport matches stable source keys and presents conflicts; it never overwrites human annotations or silently marks a changed problem mastered.
