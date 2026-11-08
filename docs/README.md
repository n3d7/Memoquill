# Memoquill Foundation documentation

M1.2 delivery: [PR #2](https://github.com/n3d7/Memoquill/pull/2) is the authoritative source-integration record for KER-98; [qualification](quality/workspace-qualification.md) records local evidence and remaining manual acceptance.

Status (2026-11-08): M2 functional X11 exit is qualified; see the [exit report](task-reports/2026-11-08-105345Z-m2-x11-exit.md) and [qualification ledger](quality/m2-qualification.md). Dev is [prepared for main](task-reports/2026-11-08-105831Z-m2-main-readiness.md); M3 does not start. Main M1–M8 remains Linux/X11-only with portable contracts; other platforms stay inactive after M8. Full X11 release AT/reflow remains M7 and deferred M1.1 manual acceptance is not passed. Full visual-design timing will be discussed after M2. [PR #3](https://github.com/n3d7/Memoquill/pull/3) is open under the later explicit publication authorization; the owner performs the merge with all commits preserved. No agent merge or release.
Foundation documentation and the M1.1 source candidate are published through [PR #1](https://github.com/n3d7/Memoquill/pull/1), following the user's explicit PR/merge instruction. See the [publication report](task-reports/2026-10-28-122447Z-foundation-publication.md); manual acceptance remains separate from source integration.

## Read in this order

1. [Foundation V1 scope](product/foundation-v1.md) and [product specification](product/product-spec.md).
2. [Learning model](product/learning-model.md), [user journeys](product/user-journeys.md), and [customization](product/customization.md).
3. [Technical design](architecture/technical-design.md), [ADRs](architecture/decisions.md), [data model](architecture/data-model.md), [ingestion](architecture/content-ingestion.md), [MCP architecture](architecture/ai-mcp-integration.md), and [detailed agent contracts](architecture/mcp-contracts.md).
4. [Verification strategy](quality/verification-strategy.md), [roadmap](planning/roadmap.md), and [Linear mapping](planning/linear-backlog.md).

## Evidence and operations

- [Actual Fedora desktop qualification](quality/fedora-desktop-baseline.md) and [M1.1 implementation/action log](planning/m1-desktop-qualification.md).
- [M1.1 review report](task-reports/2026-10-28-103031Z-m1-desktop-qualification.md), [source/artifact evidence](quality/m1-desktop-evidence.json), and [native reproduction](../tests/qualification/README.md).
- [M1.2 qualification evidence](quality/workspace-qualification.md).
- [M1.2 historical preparation](planning/m1-2-workspace-preparation.md), [execution/verification ledger](planning/m1-2-execution.md), and [implemented workspace runtime](architecture/workspace-runtime.md).
- [M2 execution](planning/m2-execution.md), [composition plan](superpowers/plans/2026-11-04-m2-composition.md), and [qualification ledger](quality/m2-qualification.md).
- [M2 qualification/startup checkpoint](task-reports/2026-11-06-101709Z-m2-qualification-and-linux-startup.md).

- [Learning science and evidence register](research/learning-science.md).
- [Activity portfolio and additional evidence](research/learning-activities.md), with [audit of the supplied Consensus review](research/consensus-review-audit.md).
- [Activity-research extension report](task-reports/2026-10-27-150554Z-learning-activity-evidence-extension.md).
- [Educational UX and accessibility](research/educational-ux.md).
- [Existing applications and inspected source](research/existing-applications.md).
- [Technical alternatives](research/technical-options.md).
- [Source register and research method](research/sources.md).
- [Third-party reuse, licenses, and attribution](architecture/third-party-reuse.md).
- [Current checkpoint](HANDOFF.md), [planning report](task-reports/2026-10-27-093441Z-foundation-research-planning.md), [agent guide](../AGENTS.md), and [project-local skills](../.agents/skills/).

The roadmap defines implementation order; Linear records its executable backlog and current tracker state. Product requirements define desired behavior, ADRs record decisions, and research documents preserve evidence and limitations. Do not infer implementation from any of them. Repository documentation is currently local until separately authorized publication.

## Architecture-first composition follow-up

- [Clarified requirements and Linear mapping](planning/lesson-composition-requirements.md): manual/agent lesson composition, replaceable UI, guest identity seam, M4/M6 boundaries and final optional M8.
- [Architecture research and evidence applicability](research/composable-learning-architecture.md): trusted registries, learning-component schemas, security/accessibility and conditional gamification evidence.

The later user instruction authorizes completing M2 and its substeps, including app launch when needed. Screenshots must be provided by the user; agents never capture them.

- [Bounded scientific screening and lesson-source requirements](research/composable-learning-screening.md): admitted versus future/rejected evidence, pinned study/assistance/metric boundaries and books/files assigned to M3/M6.

## Concrete lesson-constructor architecture

[KER-117 contract](architecture/lesson-composition.md) covers identities, typed blocks/actions, content/presentation/history boundaries, revisions/migrations, shared manual/MCP authoring and four subject traces. KER-117 is Done for M2 execution. [Evidence delta](research/lesson-contract-evidence.md) records bounded gap checks; it is reused without duplicate collection.

## X11-first delivery and future platforms

- [Portability contract and per-main-stage criteria](architecture/platform-portability.md), with ADR-10 in [decisions](architecture/decisions.md).
- [Sequencing design](superpowers/specs/2026-11-06-x11-first-platforms-design.md): planning only; no target implementation before the deferred phase.
- [Standalone later backlog](planning/linear-backlog.md): KER-122–126, after M8, inactive and unrelated to the main dependency graph.
- [Retained Wayland diagnosis](quality/wayland-deferred-diagnostic.md); historical Wayland result retained; the common X11 attribute-query case now has a tested layout workaround, not a Wayland pass.
- [Platform-planning report](task-reports/2026-11-06-193756Z-x11-first-platform-adaptation.md).
