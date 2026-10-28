# Memoquill Foundation documentation

M1.2 delivery: [PR #2](https://github.com/n3d7/Memoquill/pull/2) is the authoritative source-integration record for KER-98; [qualification](quality/workspace-qualification.md) records local evidence and remaining manual acceptance.

Status: research/product architecture/planning are preserved. M1.1 supplies the desktop qualification shell. M1.2 now implements the durable workspace and local reading preferences; local qualification is complete and source delivery is being finalized. The user authorized PR and merge after checks; manual desktop acceptance and release remain separate.

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
- [M1.2 historical preparation](planning/m1-2-workspace-preparation.md), [execution/verification ledger](planning/m1-2-execution.md), and [implemented workspace runtime](architecture/workspace-runtime.md). The later implementation request supersedes the preparation-only pause; M2 remains unstarted.

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
