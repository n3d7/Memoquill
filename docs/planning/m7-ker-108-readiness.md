# KER-108 readiness and first vertical slice

Prepared 2026-11-22 for personal use on Fedora X11. The proposed slice awaits
scope approval. [KER-108][issue] is in Backlog.

## Baseline

- GitHub main was `77d068e6dd4e7d5156fb80e257ceeda67a825c98`. [PR #9][pr9]
  merged M6.1–M6.3 at `122f1ecf`. [PR #10][pr10] added the synthetic PDF
  Journey C coverage. GitHub reported success for all four CodeQL checks
  on PR #10 head `44256b1`, whose tree matches main.
- The final M6 reports record full 595 passed, 28 skipped, 0 failed, with
  Rust doctests `not_run`, and installed native 51 passed, 0 skipped,
  0 failed. Both audits passed. The PDF journey used a synthetic profile
  and covered original-byte access and provenance. The GTK chooser remains
  unverified by that run. These results qualify the M6 candidate.
- KER-98, KER-99, KER-100, KER-101, KER-103, KER-104 and KER-107 are Done in Linear.
  KER-105 and KER-106 are also Done. Main includes the M5 package staging
  recovery code and the M6 services. The older release-oriented title in
  [the main backlog snapshot](linear-backlog.md) predates the local M5 roadmap
  amendment and the current KER-108. KER-132, KER-133, M8 and public release
  stay outside this slice.

## Provider route

Use direct streamed OpenAI Responses with Sign in with ChatGPT (SIWC) as the
first candidate. OpenAI documents ChatGPT plan usage for open-source and locally
hosted clients, including dynamic registration, browser authorization with a
loopback callback and PKCE, ID-token validation, and a separate inference scope.
The preview uses `store: false`, `stream: true` and application-managed history
for HTTP. Client-side typed function tools are supported within the preview's
documented format. [Overview][siwc] · [Sign-in][sign-in] · [Preview limits][limits]

Keep identity, refresh, token rotation and revocation in the native backend.
Use native secure storage per account registration, and show the active account,
model and billing mode in the renderer. Use one active account at a time.
Distinguish confirmed remote revocation from local disconnect. Store the local
conversation independently from the renewable provider session.
[Accounts and sessions][sessions]

OpenAI also documents App Server as an optional wrapper using the app's OAuth
access token. Direct Responses leaves chat history and the tool loop in
Memoquill and avoids introducing a runtime with general-purpose tools. If
eligibility or required functions fail on the intended account, qualify the
documented platform API-key flow with separate billing, subject to approval.
App Server's managed login protocol alone does not establish entitlement for
Memoquill. [App Server][app-server] · [Function calling][functions] ·
[Streaming][streaming] · [Separate billing][billing]

Before claiming SIWC support, verify a consented synthetic turn on the intended
account, plan, workspace and model. Check registration, granted inference
scope, model inference, an application tool round trip, cancellation, refresh,
reconnect and sign-out. Verify that the intended Fedora environment offers
secure credential storage. A model catalog entry is insufficient evidence of
entitlement. Keep provider account access optional while this evidence is pending.

## First slice

From a seeded course, connect one provider, open a local course thread, select
one existing lesson and a bounded source excerpt, and send a request to find
and open the lesson. Ask the assistant to propose one cited addition to its
composition. Inspect and apply or reject the draft in the existing M6 lesson
review. Exercise the same path with a deterministic provider fixture.

| KER-108 criterion | Implementation seam |
| --- | --- |
| Connect, identify, reconnect and disconnect | Native provider adapter with a secure account store and typed connection status. Use an ephemeral loopback listener only during approved sign-in. |
| Chat, context and continuity | Course-scoped threads and ordered run events. Store locally, choose explicit course, lesson and source revision context, and show which prior messages and tool results will be sent. Stream and stop through typed desktop events. Explicit restart creates a new run with fresh authority. |
| Typed reads and navigation | Reuse `CourseReads` for bounded `Search`, `Lesson` and `Source` projections. Check selected course, source and revision before disclosure. Open a validated target in existing lesson tabs. The model cannot set the course, actor or grant. |
| Draft and review | Use the M6 lesson draft service with native draft ownership and stable operation IDs. Pin published base, source revision and locator. Hand the draft to `ExternalDraftReview` and `lesson_review_external_draft`. Apply or reject in the trusted desktop using the exact revision and preview hash. |
| Grant lifetime and offline use | Bind each run to its account, course, operation set and context. The M6 facade captures a course service. Add run-level checks for reads and proposals, and carry revocation to queued operations. Keep networking and provider state outside the learning service. |

The initial agent can search within the approved scope, request navigation and
propose one lesson change. Existing learner-edited material requires affirmative
permission before it becomes a draft target. Applying a proposal requires a
separate confirmation. The in-app agent receives no delegation, original-file
bytes, private learner records or general execution tools. Treat imported
material, model output and tool arguments as untrusted. A tool result sent back
to the model is a disclosure and must pass the same scope checks as the prompt.

## Acceptance checks

1. With a deterministic provider, stream in order, record one terminal state,
   and retain the thread after reopening its course. Stop, switch course or
   disconnect during streaming and verify that late output and tool requests
   are dropped. An interrupted run stays inert until the learner approves a
   restart. Prior local threads remain readable after disconnect.
2. Use two courses and test forged IDs, stale lesson/source revisions, revoked
   grants and a cross-account switch. Verify that hidden data never enters
   outbound payloads, events or model-visible error details. Test an imported
   instruction that requests an operation the learner has not approved.
3. Inspect a draft with a pinned citation, confirm the exact preview, and verify
   one durable receipt. Test rejection, a changed lesson head, concurrent
   confirmation, a lost response and retry with the same operation ID. Preserve
   prior lesson revisions, original bytes, source provenance and learning records.
4. With provider unavailable, verify ordinary course navigation, editing, study,
   search, progress and offline recovery. Keep native credentials out of exports
   and backups. Test restore behavior for the chosen chat-history policy.
5. Run the approved live-provider smoke with a synthetic course and review the
   outbound context. Qualify account, billing label, selected model, stream,
   one tool call, Stop, refresh/reconnect and sign-out on the intended
   account. Keep deterministic results and live evidence distinct.

The verification runner is local tooling and absent from the main checkout.
Stage the known M6 runner locally in the isolated implementation checkout and
use `python3 tools/verification/verify.py run --profile affected`. Use a
targeted `--rust TEST_NAME`, `--frontend PATH` or `--only CHECK` for a concrete
regression and `--profile full` for the final candidate. Read the final JSON,
use the failing check's log, and use `audit` to recheck evidence. Record
`passed`, `skipped` and `not_run` separately. Launch the GUI only with explicit
authorization, in isolated Podman X11 with SELinux Enforcing and no screenshots.

## Decisions before implementation

1. Approve the single-provider slice with one lesson proposal and manual
   review. Confirm whether the API-billed fallback may be qualified if SIWC
   is unavailable, and identify the intended account/workspace for the later
   synthetic smoke without sharing secrets.
2. Agree which chat history to carry in course-native recovery and whether
   personal chat export/clear must ship in the first slice. Default proposal:
   include chat in native course recovery, exclude it from shareable course
   packages, and keep clearing chat independent from learning history and receipts.
3. Confirm disclosure consent for the displayed recent history and explicitly
   selected lesson/source context. Propose separate confirmation before
   reusing a thread with another provider account.

[issue]: https://linear.app/kerigan/issue/KER-108/m7-deliver-in-app-ai-account-and-agent-workspace
[pr9]: https://github.com/n3d7/Memoquill/pull/9
[pr10]: https://github.com/n3d7/Memoquill/pull/10
[siwc]: https://developers.openai.com/siwc/token-sharing-open-source
[sign-in]: https://developers.openai.com/siwc/token-sharing-open-source/sign-in
[limits]: https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations
[sessions]: https://developers.openai.com/siwc/token-sharing-open-source/profiles-and-sessions
[app-server]: https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server
[functions]: https://developers.openai.com/api/docs/guides/function-calling
[streaming]: https://developers.openai.com/api/docs/guides/streaming-responses
[billing]: https://help.openai.com/en/articles/8156019
