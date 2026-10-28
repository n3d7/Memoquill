# Personalization without configuration burden

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
