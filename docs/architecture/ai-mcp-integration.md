# Optional external AI and MCP

## Specification and scope

Live official documentation checked on 2026-10-27 identifies **2026-07-28** as the current MCP revision. It uses per-request version/identity/capability metadata rather than the legacy connection-wide initialization handshake. Earlier clients through 2025-11-25 use the legacy era; dual-era behavior must be tested explicitly. Do not implement a guessed 2025 handshake and call it current. [Official versioning](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning).

Use the official Rust SDK (`rmcp`) as a protocol adapter after pinning a released version/commit with verified features and actual conformance; do not hand-write a JSON-RPC stack. Implement the core stdio surface and compatible modern/legacy behavior supported by that pin. No Tasks, sampling, elicitation dependency, MCP Apps UI, remote transport, or AI-provider SDK is needed for V1. The application contracts below remain independent of protocol revision and host brand. [Official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk).

## Local transport and ownership

Choose **stdio** for clients such as Codex, Claude tools, and local hosts. A packaged `memoquill mcp` mode is launched by the host, communicates only protocol on stdout/stdin, and sends redacted diagnostics to stderr. Stdio is a standard transport; it avoids exposing an HTTP listener. [Official stdio binding](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio).

The bridge forwards bounded typed commands over a Unix-domain socket to the running desktop application's broker. The desktop process is the sole owner of writes/database/assets. Use an XDG runtime directory with owner-only access, peer UID checks, and a scoped random grant; peer UID or self-reported client identity alone is insufficient authorization. This private IPC is an implementation detail, not an unrestricted second MCP API. Stale sockets/process loss fail closed. When the app is closed, return an actionable `app_not_running`; no persistent daemon or hidden database owner is started.

The user enables Connections, pairs a requesting bridge, sees its claimed label/process context and requested data/operations, and approves a grant. A one-time challenge ties the UI request to the bridge instance; persistent grants are optional, revocable, and expire as configured. Store only a token hash in application grant records; a launcher-side secret, if persisted, uses a private runtime/config file or desktop secret store and never command arguments, logs, exports, or repository files. A token is scoped to a workspace/subject/resource set and capabilities, not a generic application password.

Trust boundary: this constrains access through Memoquill services. It cannot defend against a fully compromised OS account/host with independent access to that user's files, nor control where an authorized external host sends material after reading it. The grant UI explicitly treats source/progress reads as disclosure to the chosen host. No claim of model-level exfiltration prevention is made.

## V1 application capability surface

Version application envelopes/schemas independently (initial contract version 1), use stable IDs, cursor pagination, bounded byte/item counts, and structured errors. The following are application tool families, not a replacement for MCP's transport/versioning methods. The detailed [agent contracts](mcp-contracts.md) define composable authoring, inspection, previews, jobs, autonomy and error recovery. The goal is a powerful product client, not a read-only add-on or a single “generate cards” endpoint.

| Tool / resource operation | Authority and behavior |
| --- | --- |
| `memoquill_capabilities` | Reports contract version, supported types/limits and granted scopes; no private enumeration before pairing. |
| `memoquill_subjects_list`, `memoquill_structure_get` | Read selected subjects/collections/units/paths/objectives and revisions; no hidden subjects in counts/errors. |
| `memoquill_content_search` | Search within grant; filter/paginate, return attributable IDs/snippets/revisions. |
| `memoquill_resource_read` | Read selected normalized content, metadata, source anchors or bounded original chunks through opaque IDs. No arbitrary path/URI or implicit network fetch. |
| `memoquill_progress_get` | Separate explicit progress-read permission; summarized evidence by default, raw responses only if specifically granted. |
| `memoquill_import_stage` | Submit bounded prepared structured records/bytes or use an existing user-selected staged handle. Same validation/rights/provenance pipeline; no arbitrary local files/URLs. |
| `memoquill_draft_create`, `memoquill_draft_edit`, `memoquill_draft_read` | Persistent agent work area with typed creation/revision/organization of subjects, goals, collections, units, resources, paths, activities and links; inspect its effective overlay and continue across host restarts. No manual approval for each draft edit. |
| `memoquill_proposal_submit`, `memoquill_proposal_validate`, `memoquill_preview` | Produce a reviewable version of a draft, run schema/reference/organization/accessibility checks, and inspect inert lesson/activity/session view models. Validation is not scientific or answer-truth verification. |
| `memoquill_proposal_apply` | Apply only under an exact trusted-UI approval or a matching bounded autonomy grant. Default is review; the agent cannot grant authority to itself. |
| `memoquill_proposal_status`, `memoquill_job_status`, `memoquill_operation_get`, `memoquill_operation_undo` | Resume jobs, inspect scoped receipts/diffs and request safe compensating undo. Later edits/history, scope or budget conflicts require a refreshed proposal/review. |

MCP resources can expose the same scoped content through opaque `memoquill://...` identifiers with correct MIME/size metadata. Tool/resource permission checks share the application policy; resource reads are not a less-restricted side door. Gap/duplicate analysis and personalized planning are performed by the external host using authorized reads/proposals, not by a built-in provider dependency.

No V1 tool grants itself permissions, deletes history, changes grades/review events, runs code, executes SQL/shell, browses the filesystem, or makes arbitrary network requests. Live mutation uses the same validated transaction service as the UI and requires authority created by the trusted UI. Agents may propose personalized session/practice plans and draft feedback without fabricating learner attempts or competence.

## Useful autonomy, explicit delegation

Default mode permits extensive reads and draft work, with UI review before live application. The user may instead grant time/budget/scope-limited **autonomous reversible authoring**: create new content and structure in a selected subject/collection, revise agent-owned content not subsequently human-edited, or organize/tag/link existing material when that operation class was explicitly delegated. Such actions require no per-operation click, but still undergo validation, revision checks, audit and resource limits.

Destructive replacement/merge, overwriting human-edited answers/lessons, cross-scope changes, source removal, private-data export beyond the read grant, and other sensitive changes always require a specific review. Protected semantic changes to agent-owned material that already has learning evidence require review or an explicit prior user-selected relevance policy; ownership alone cannot silently invalidate evidence. History/grades/grants cannot be edited through a V1 agent tool. “AI-generated/unverified” provenance remains even for auto-applied content. Optional activation does not mean weak capability: large-course authoring and iterative correction should be possible through ordinary composable services.

## Proposal transaction

1. Agent reads current IDs/revisions and declared activity/document schemas.
2. Submission includes contract version, workspace/scope, idempotency key, base entity revisions, typed operations, source references, authorship/model/host metadata as unverified claims, and intended rationale.
3. Validate sizes/types, references, schema versions, rights metadata, prerequisite/containment cycles, grant scope and conflicts. Return field-level errors, warnings, affected counts, and a deterministic proposal hash.
4. Review mode displays additions/edits/moves, source links, unsupported/uncited claims and learning-history implications. User may reject/edit/select a subset; editing produces a new validation/hash. Autonomous mode deterministically checks the operation class, target scope, ownership/human-edit status and remaining delegation budget; ambiguous or sensitive work returns `approval_required` rather than expanding permission.
5. Specific approval binds the exact selected operation hash and base revisions; delegated application binds the exact validated hash to the active policy version/budget. Recheck permission, revocation, conflicts and semantic invariants **inside the apply transaction**. A passed validation earlier is not sufficient.
6. Commit revisions/placements/relations and audit as one operation; preserve originals, prior revisions, annotations and attempts. Return operation ID/status. Retry is idempotent; failure has no partial live application.

There is no `approved=true` parameter an agent can supply. Approval and autonomy-grant records are created only by the trusted UI path. Undo is a compensating operation subject to current authority and subsequent edits/history, not blind rollback of everything that happened since.

## Threats and controls

| Threat | Deterministic control / remaining limit |
| --- | --- |
| Malicious imported instructions / prompt injection | Content returned as labeled untrusted data; permissions checked independently of text/model decisions; no authority inferred from documents or agent rationale. Cannot guarantee host/model resistance. |
| Scope leaks / guessed IDs / confused deputy | Authorize every operation, nested reference, snippet, resource chunk, progress field and effective shared-object impact. A granted placement cannot mutate denied placements through their common target; require explicit shared-object authority, fork or review. Uniform denied/not-found behavior; no hidden names/count leakage. No downstream credentials/token passthrough. |
| Self-approval / races / stale edits | UI-owned approval/delegation policy, exact hash/policy version, transactional revision/revocation/budget checks, idempotency and audit. |
| Exfiltration / network access | No server network tools/fetching; explicit consent to host reads. Host/cloud use after disclosure remains outside enforcement. |
| Resource exhaustion | Proposed caps: 2 MiB request, 256 KiB resource chunks, 100 listed entries, 50 search hits, 500 operations per proposal, bounded worker/time/concurrency and cancellation. Test/revise budgets, never accept unlimited allocation. |
| Path traversal / execution / malicious renderer | IDs/staged handles only, schema/URL/node allowlists, passive original handling, no code/macros/scripts, no invoke privileges in imported surfaces. |
| Grant theft / logs | Private IPC/secret storage, short/revocable grants, hashes, no secrets/content in ordinary logs. Same-user OS compromise remains outside service boundary. |

The official security guidance separately warns about local server installation/proxy risks and authorization pitfalls; MCP itself does not enforce application consent or safe model behavior. [MCP security guidance](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices).

## Qualification and release boundary

M6 must test modern and a supported legacy host path, permissions on all tools/resources, deep draft authoring/resumption, user-delegated autonomous application and escalation, untrusted content, revocation during a request, stale proposals, atomic apply/undo conflicts, cancellation/limits, app shutdown, and no-listener offline behavior. Use SDK conformance tests plus an actual external-client course-authoring workflow. Pin implementation/spec fixtures and document unsupported revisions; a working protocol handshake alone is insufficient. V1 enables externally orchestrated curriculum work with explicit authority; it does not include a built-in autonomous model/provider or guarantee AI explanations are correct.
