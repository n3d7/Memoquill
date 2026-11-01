# Lesson composition: architecture-first requirements

Status: user requirement clarification and planned work, 2026-11-01. This is not an approved implementation design or a claim that the constructor exists. Application feature/UI development and further GUI qualification are paused until the revised architecture is agreed. Existing application commits remain evidence for their earlier bounded scope.

## Phase mapping

| Task | Responsibility |
| --- | --- |
| [KER-117](https://linear.app/kerigan/issue/KER-117) | Resolve lesson ownership against existing unit/resource/activity concepts; freeze content/activity/composition/history separation, versions, references and trusted component/action capabilities. |
| [KER-118](https://linear.app/kerigan/issue/KER-118) | Replaceable presentation, controllers, narrow application ports and native/test adapters; shared controls/design tokens; guest identity/session seam for future login. |
| [KER-119](https://linear.app/kerigan/issue/KER-119) | Validated composition persistence, semantic proposals/patches, transaction/revision/receipt behavior and offline migration/recovery. |
| [KER-120](https://linear.app/kerigan/issue/KER-120) | Manual offline lesson-tab composer and trusted shared renderer/preview, with keyboard/focus/reading-order and isolation checks. |
| [KER-116](https://linear.app/kerigan/issue/KER-116) | Expanded integrated M2 acceptance after those foundations; current tests/screenshots do not establish the new scope. |
| [KER-102](https://linear.app/kerigan/issue/KER-102), [KER-104](https://linear.app/kerigan/issue/KER-104) | Durable study behavior, evaluation, help evidence and honest outcome-specific progress. Keep preview and reward state separate. |
| [KER-105](https://linear.app/kerigan/issue/KER-105), [KER-106](https://linear.app/kerigan/issue/KER-106), [KER-107](https://linear.app/kerigan/issue/KER-107) | Shared-service MCP discovery, manual/agent authoring parity and enforced lesson/action authority with review or explicit bounded delegation. |
| [KER-121](https://linear.app/kerigan/issue/KER-121) / M8 | Final optional per-lesson gamification after core M7 qualification: configurable achievements/levels/metrics/rules, default off and evidence-calibrated. |

Dependency: KER-117 precedes KER-118/119; both precede KER-120; KER-116 depends on all four. KER-121 follows KER-108 and does not block the core M7 stage. Completed KER-109–112 slices are point-in-time evidence, not approval of this revised architecture.

## Required boundaries

- A person can compose their own learning methods from content, activities/tests, containers and interaction/presentation primitives. The agent gets the same semantic operations later; a mandatory pedagogy/template is not imposed.
- Store canonical content/provenance, versioned activity definitions, composition/presentation and personal learning outcomes separately. Preserve IDs, anchors, revisions and history across recomposition; define stale references and migration explicitly.
- Presentational views use view models/actions. Controllers own scenarios/drafts/retries; native IPC is one adapter behind a frontend application port. Rust services retain invariant, permission, transaction and storage authority. Replacing a representative screen/design system must not require changing Rust storage/domain rules.
- A versioned registry supplies trusted component implementations, typed prop/slot/action/event contracts and accessibility/fallback behavior. User/agent payloads are bounded declarative data, not arbitrary code or unrestricted CSS/network/filesystem requests.
- Each lesson tab owns its local composition/settings/state. Cross-lesson/global changes and private outcomes require separate authority. Preview uses the same renderer as the lesson but has separate outcomes from a real study session.
- A future identity/session adapter starts with a local guest actor. Actual registration/provider/password/session design is deferred. Keep account identity distinct from agent grants and do not add fake login UI or mandatory cloud/account dependencies.
- Optional M8 metrics explain their denominator, rules and provenance; completion/rewards are separate from independently demonstrated competence. Users can create safe experimental methods; recommended presets carry applicable evidence and benefit/cost checks.

## Open decisions before implementation resumes

Resolve lesson aggregate ownership, initial activity/component vocabulary and interaction semantics; document serialization/reference/version/fallback and resource budgets; choose the minimal manual vertical slice; specify state/attempt ownership and supported extension trust. Compare an owned renderer with a json-render adapter in a separately approved bounded spike if needed. No new renderer dependency or auth provider has been chosen.

Architecture agreement and an implementation plan come before application feature changes. Preserve the existing offline, untrusted-input, source-provenance, physical GNOME/human Orca and release authorization gates. See [research and source applicability](../research/composable-learning-architecture.md).
