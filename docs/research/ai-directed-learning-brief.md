# AI-directed learning: provisional user brief and evidence delta

2026-11-08. Requirements discussion after M2; no application implementation,
approved architecture change or M3 start. UI design remains deferred.

## User intent and remaining decisions

The user wants the application to take as many learning decisions as useful,
with an external AI adapting the experience and checking work through MCP.
The learner mainly completes lessons/tasks. They explicitly describe this as
provisional: research should determine productive delegation, rather than a
fixed conventional learning-app design. Optimize useful learning per time/effort,
not feature count or unrestricted automation. No universal optimum is established.

Subsequent clarification: any external model/provider may connect through MCP;
no provider/tariff or budget answer is an application prerequisite. The learner
sets the goal/focus; AI gathers sources and progressively adapts courses. Each
sidebar tab is an entire isolated course with lessons inside, not an individual
lesson. The user selected a separate post-M2 step, [KER-127](../planning/course-workspace-followup.md).
Host availability and disputed/uncertain-assessment policy remain open; do not
silently settle them as user choices.

## Reused evidence and existing contracts

- [Learning model](../product/learning-model.md): outcome-specific evidence,
  assisted versus independent attempts, spacing/retrieval eligibility and transfer.
- [Activity evidence A9–A14](learning-activities.md): AI tutoring interventions
  differ; short-term assisted success is not durable independent competence.
  Existing publisher/arXiv reads were not repeated.
- [MCP contracts](../architecture/mcp-contracts.md) already allow substantial
  delegated reversible authoring. Existing attempts/grades stay immutable;
  current V1 has no arbitrary agent grade/history mutation. A future AI-assessment
  proposal must distinguish feedback/judgment from the learner's observed attempt
  and authoritative competence claims. This brief does not change that contract.

## New bounded grading evidence

[Korthals et al., 2026](https://www.mdpi.com/2504-4990/8/3/74),
DOI10.3390/make8030074: publisher methods, selected results and limitations read.
The dataset has46 graduate students,130 coding/open questions and five assignments.
Rubrics were revised; the first assignment tuned thresholds and four were held
out. Repeated model scoring plus simulated selective human regrading was studied.
Single-prompt scores were biased; repetition did not remove all bias. Human
scores are a pragmatic reference, not infallible truth.20 samples per answer
also incur cost/latency. This is grading-agreement evidence, not a study proving
durable learning or universally reliable autonomous grading across subjects.

Provisional product implication: evaluate grading errors by task/rubric/model,
retain assessment provenance and disputed/unknown outcomes, and test independent
learning separately. A model's self-confidence or repeated agreement is not a
validated universal mastery signal. Multi-sampling/human review are study
mechanisms, not automatically adopted product requirements.

## Retrieval limits and next step

Two Consensus searches; selected grading record fetched after correcting an
invalid URL-as-ID request. Most tutor hits overlapped existing evidence.
Native arXiv exact-title search returned0; this does not prove absence.
Firecrawl Cloud located/read one publisher page: observed991 credits before
retrieval, search2 and scrape1 reported, search feedback refunded1. No purchase.
Full paper was retrieved but only the stated sections appraised; not a systematic
review. No private lessons/history were transmitted or paper text committed.

Next: clarify course identity and operational/assessment behavior, then scope
remaining evidence gaps before proposing a design. The later course clarification
is tracked separately; this brief does not authorize runtime changes.
