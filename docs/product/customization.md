# Personalization without configuration burden

> Current architecture clarification: see the composition section below. New feature/UI development is on hold pending architectural agreement.

Defaults support immediate reading/practice. Customize through progressively disclosed settings, not a setup wizard requiring all preferences in advance. Interface choices are engineering/UX proposals informed by [educational-ux.md](../research/educational-ux.md), not prescriptions derived from learning-style categories.

| Layer | Controls and behavior |
| --- | --- |
| Workspace | UI locale, system/light/dark theme, font/reading width, reduced motion, keyboard bindings, navigation/view preferences, backup location/policy, connections. |
| Subject | Goals/prior-knowledge notes, default path, preferred activity mix and session budget, recall scheduling overrides where meaningful. Inherit workspace defaults and show overrides explicitly. |
| Session | Scope, duration/item budget, new/review/practice mix, hints/feedback policy, blocked/mixed selection, pause. Temporary choices do not silently overwrite subject defaults. |
| Resource/activity | Reading position, captions/alt text, activity assessment policy, recall eligibility, explicit difficulty/rubric. No arbitrary “difficulty” slider pretending to measure ability. |

Use stable action IDs for shortcuts, detect collisions/reserved OS keys, and keep menu/button alternatives and an accessible reset route. System theme follows desktop preference with explicit override. Text scaling must reflow normal prose; code/equations/tables can scroll without hiding surrounding controls. Do not promise one universally optimal font or theme. Custom palettes require validation of text, non-text, error, disabled, and focus states before activation; fall back to accessible built-ins if invalid.

Saved views, recent places, tags, breadcrumbs, and persistent search filters reduce repeated organization. Templates are optional reusable collection/activity presets, never fixed subject categories. A template copies structure with new IDs and documented source attribution; it does not clone someone else's progress. Import mapping presets are conditional on a real repeated workflow, not an open-ended automation system.

Keep advanced scheduler parameters, provenance, and diagnostics available but outside the primary learning pane. Show save/due/feedback/review status clearly. Avoid streak pressure, loot systems, forced animations, or engagement-based claims of learning. Record calibration/confidence optionally; skip it when it adds more burden than value.

Acceptance includes journey F, keyboard-only operation, reset without data loss, per-subject inheritance tests, and a small usability evaluation of novices and experienced collection users. Preferences must be versioned and exportable; adding a setting requires a safe default/migration and does not make old workspaces unusable.

<!-- lesson-composition-clarification:start -->
## Learner-built lesson experiences — clarification, 2026-11-01

A learner can compose a lesson from content, exercises/tests, interaction and presentation blocks, arrange/nest/configure them, and create safe experimental learning methods. This is first-class manual offline authoring. Later MCP authoring has the same semantic capabilities and constraints.

Each lesson tab owns its composition and local settings. Lesson changes do not restyle the global shell, alter another tab, erase source/content revisions or replace personal learning history. Provide explicit inheritance/overrides for ordinary appearance settings, meaningful reading order, keyboard alternatives, undo/recovery and a shared author-preview/lesson renderer with separate outcome state.

Recommended learning methods and app-specific agent authoring guidance use applicable research, clear objectives and appropriate practice/feedback/outcome checks. Preserve user choice; distinguish evidence-backed recommendations from custom experiments. Optimize useful demonstrated learning per learner time/effort, not clicks, app time or reward totals.

Optional gamification belongs to M8/KER-121 and is off by default for each lesson. People can attach/configure achievements, levels, scores/ratings, percentages and scales through bounded versioned rules. No global XP/account level or automatic cross-lesson effects. Explain thresholds/denominators/provenance, distinguish rewards/completion from competence, allow easy opt-out and explicit reset/history behavior, and retain ordinary study data.

Future registration/login has a guest-first identity/session seam in KER-118. Actual account/provider/UI/service work remains deferred; core learning has no account/network requirement. Do not expose nonfunctional registration controls.

These are clarified requirements; current M2 fixed forms are not the finished constructor. Implementation is held for revised architecture agreement. See [phase mapping](../planning/lesson-composition-requirements.md).
<!-- lesson-composition-clarification:end -->
