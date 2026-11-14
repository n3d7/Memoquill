# Foundation V1 implementation roadmap

Current user decision,2026-11-06: the main M1–M8 cycle builds the working application
for Linux/X11 only. Future Windows, macOS, Linux/Wayland, iPhone/iOS and Android
work is standalone and inactive after that cycle. Every main stage still preserves
its explicit portability boundary. See the [platform contract](../architecture/platform-portability.md)
and ADR-10 in [decisions](../architecture/decisions.md).

Current continuation (2026-11-13): M3 PR #5 is merged; KER-100 and KER-101 are
Done, confirmed by Linear readback. M2 and post-M2 course isolation KER-127–130
are also Done. PR #6 separately removed local agent instructions/skills from
tracking while retaining local files. KER-97 is Backlog, with remaining physical
Fedora/X11/human Orca acceptance deferred to M7; this is not a passed gate.
M4.1 and M4.2 implementation is complete locally and a PR into main is authorized.
See [M4 execution](m4-execution.md) and the M4 delivery/GUI receipts for actual
qualification results and explicitly omitted release gates.
M2 PR #3 was merged by the owner. Course-level isolation is a separate post-M2 step, not a reopened M2.
[Requirements and substep map](learning-productivity-followup.md) owns the latest
course/AI/manual-edit/research-schedule clarification. Full visual design remains
deferred. No GUI, schedules or publication started in this continuation review.

Core Foundation spans M1–M7. M8 remains the final optional lesson-local extension
and does not block the core M7 release. Other platforms do not start automatically
at M7 or M8 completion. No main issue depends on the deferred platform tasks.

| Order | Milestone | Dependencies | X11 capability / exit | Portability preserved in this stage |
| --- | --- | --- | --- | --- |
| M1 | Desktop foundation and durable services | None | Supported Fedora X11 RPM/runtime, typed services, durable SQLite/assets/migrations and safe failure. KER-97's manual evidence remains unaccepted; the user-approved development deferral is distinct from M7 release acceptance. | Native roots, locking, lifecycle and errors behind the application/native boundary; record Linux assumptions. |
| M2 | Composable lessons, authoring and navigation | M1 service/storage baseline | Organization, canonical editing, passive originals/search and KER-117–120: trusted composition contracts, replaceable UI/controllers, durable drafts/receipts, isolated lessons and offline composer/preview. Qualify current X11 functionality; no other-platform matrix. | Stable lesson/content/activity/history formats; no X11/window identity in domain data; typed UI/IPC/service seams. |
| Post-M2 | Separate course-workspace foundation (KER-127) | M2 integrated | Course identity/migration (128), functional isolation/nested navigation (129), versioned future interactive-extension contracts (130). Settle course contracts before dependent implementation; no simulator runtime or full visual redesign. | Course-scoped portable IDs, services/state/capabilities; current X11 only. |
| M3 | Lesson sources, ingestion, discovery and ownership | M2 | KER-100 preserved research/books/files, qualified PDF/EPUB/common-format intake, locators/coverage, safe reimport/search; KER-101 package/recovery completeness. Retain PDF/resource/security gates. | Provider/file selection and native safe I/O separated; immutable bytes and portable manifests/IDs, versioned package round-trips without absolute host paths. |
| M4 | Activities, feedback, sessions and recall review | M2 | Four families, exactly-once durable attempts/drafts, feedback/help, scoped assessments, FSRS recall adapter and separate practice reminders; extend recovery with history. | UI-independent learning services and checkpoints; isolate clocks and process/window lifecycle from study policy. |
| M5 | Progress, personalization and workflow completion | M3,M4 | Honest outcome-specific history/coverage/errors, accessible settings/navigation and consistent offline resumption; no universal mastery percentage. | Semantic commands/view models and logical settings; input/layout/window details cannot become learning state. |
| M6 | Capable MCP authoring, scoped autonomy and reviewed changes | M3,M5 | KER-105–107: desktop stdio bridge, shared scoped services, discovery/schema/reads, durable drafts/previews/jobs, grants/revocation and review or delegated reversible apply/audit. | Separate desktop transport/native authority from portable services/DTOs; explicit capabilities, not an assumption that mobile can spawn stdio/server processes. |
| M7 | Fedora X11 release qualification | M1–M6 | Supported RPM, X11 offline/keyboard/IME/focus/reflow/assistive-technology, scale/security/crash/migration/recovery, four subjects, reviewed examples and notices. Local checks do not authorize publication. | Audit native assumptions and produce the contract/adapter/capability/test handoff for later ports; do not run foreign-target builds now. |
| M8 | Optional per-lesson gamification | M7;shared M4–M6 contracts | KER-121: default-off lesson-local achievements, levels, scores/ratings, percentages/scales and bounded custom rules; transparent metrics and evidence. X11 only; no global XP/account level. | Versioned UI-independent rules/metrics; no platform data fork or linked mutable lessons. |

Readiness is a design/acceptance criterion within normal X11 work, not a separate
multi-platform implementation project. Keep existing native implementations until
their replacement is needed; do not add unused adapters, SDKs or speculative crates.
The [stage/issue matrix](../architecture/platform-portability.md) specifies the
required boundary in each main issue. Data/schema/service checks still run on the
current supported environment; they are not Windows/mobile qualification.

M3 and M4 can proceed independently after M2 and the applicable course-boundary
contracts when authorized and ownership is
separate; M5 joins them. MCP contracts are considered early but working external
transport follows stable user workflows in M6. Never expose raw storage as a shortcut.
The [learning-activity evidence](../research/learning-activities.md) continues to
refine M4/M5/M6 without changing the four-family architecture or inventing subject
special cases, a drawing engine or universal pedagogical lockouts.

## Qualification decisions

- M1's actual packaged baseline and deferred manual evidence remain distinct. The
  earlier physical GNOME/Wayland gate is superseded as a main-development prerequisite
  by the user's X11-first decision; failures and unrun tests are retained, not passed.
- Core X11 keyboard/semantic/focus, offline, data integrity, recovery, security and
  licensing requirements remain. Full X11 assistive-technology/reflow acceptance
  belongs in M7 before a supported release claim; KER-97 remains Backlog.
- M3's conditional PDF gate still requires passive licensed/resource-bounded
  extraction/viewing and anchors. Failure reduces capability honestly.
- M4 preserves history across revisions/schedule changes. FSRS applies only to
  recall; algorithm tests are not learning-efficacy evidence.
- M6 requires useful authoring and service-level scopes, default review or explicit
  bounded delegation. No unrestricted listener, self-grant or direct DB fallback.
- Non-X11 failures are tracked in the deferred platform work. Common data/security
  defects or failures on supported X11 remain main-cycle issues.

## Deferred standalone platform work

Milestone: **Later — platform ports after M8 (inactive)**. Backlog, no parent/core
blocking or related links, no dates/cycles/automatic execution:

| Issue | Future target |
| --- | --- |
| KER-122 | Windows desktop |
| KER-123 | macOS desktop |
| KER-124 | Linux/Wayland |
| KER-125 | iPhone/iOS |
| KER-126 | Android |

Detailed boundaries and links: [portability contract](../architecture/platform-portability.md).
Current Wayland failure evidence is [retained separately](../quality/wayland-deferred-diagnostic.md).
The future interactive-extension foundation is planned separately; executable
runtimes/simulator implementations still require an approved design.
Cloud/sync/accounts/collaboration, autonomous in-app AI, executable sandboxes,
unqualified OCR/complex formats, full web crawling, advanced psychometrics and a
marketplace remain outside the core cycle. New ideas require explicit scope review.

## Historical clarification records

The November1 architecture hold was superseded by the November4 execution authority
and completed KER-117 contracts; the November6 pause was superseded by resumed M2 work and its merged PR.
The November8 course foundation is now complete; the November9 continuation checks the next M3 step. Historical research/reports remain evidence, not current execution policy.
The [lesson requirements](lesson-composition-requirements.md), [source admission](../research/composable-learning-screening.md)
and [execution ledger](m2-execution.md) retain their valid ownership, science,
source/provenance and subject-independence decisions.
