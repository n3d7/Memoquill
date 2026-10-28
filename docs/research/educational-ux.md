# Educational interface evidence and qualification

Status: Foundation research and proposed requirements; no interface has been implemented or qualified.
Sources checked: 2026-10-27. Scope: an offline Linux desktop application for learning arbitrary subjects.
English, Python, mathematics, and cooking are validation examples, not privileged content types.

## How to read this document

- **E — empirical:** a reported experiment; population, task, outcome, and limits matter.
- **S — standard:** a normative accessibility requirement, with its stated scope and exceptions.
- **H — heuristic:** established design guidance; useful for inspection, not proof of a learning effect.
- **P — proposal:** a Memoquill engineering hypothesis or acceptance gate requiring evaluation.

Usability, accessibility, preference, and learning effectiveness are different outcomes. Faster navigation
does not demonstrate better retention or transfer. Reduce effort spent operating the application while
preserving intentional learning effort, such as retrieving an answer before revealing it.
All numeric usability/performance targets below are provisional engineering gates, not empirical guarantees.

## Evidence and its limits

**E1 — Learner pacing.** Mayer and Chandler (2001) report two experiments with narrated animations
explaining lightning. Learner-paced presentations improved transfer in the relevant comparisons, without
the corresponding retention advantage. This supports evaluating pause/advance controls for explanations;
it does not establish an optimal session duration, unrestricted learner control, or benefits for every subject.
[Primary paper](https://tecfa.unige.ch/tecfa/teaching/methodo/Mayer_Chandler01.pdf),
[publication record](https://eric.ed.gov/?id=EJ638751).

**E2 — Reading presentation.** Rello, Pielot, and Marcos (CHI 2016) used eye tracking and reading tasks
with 104 participants, Wikipedia articles, Arial, several font sizes, and several line spacings. Larger sizes
improved the reported readability measure; comprehension was better at 18 and 26 points in their conditions.
The experiment does not establish a universal desktop font, literal CSS size, or optimal mathematics/code layout.
Support resizing and test comprehension instead of declaring one typography setting scientifically optimal.
[Author-hosted primary paper](https://pielot.org/pubs/Rello2016-Fontsize.pdf).

**E3 — Display polarity.** Piepenbrock, Mayr, and Buchner (2014) found better proofreading performance
and smaller pupils with dark text on a light background under the tested conditions. This is evidence about
a bounded reading task, not universal preference, night-time comfort, every visual impairment, or long-term
learning. Only the abstract was verified here; effect sizes and the complete experimental protocol were not.
[Primary study abstract and DOI](https://pubmed.ncbi.nlm.nih.gov/25135324/).

**E4 — Resuming interruptions.** Trafton et al. (2003) experimentally compared an eight-second warning
before interruption with immediate interruption. Preparation was associated with faster task resumption;
participants also adapted with practice. Saving context and showing a resumption cue are product inferences,
not interventions directly tested by that study or evidence of improved learning retention.
[Primary paper](https://www.interruptions.net/literature/Trafton-IJHCS03.pdf).

**H1 — Usability inspection.** Nielsen's heuristics motivate visible status, recognizable choices,
consistent terms, user control, error prevention/recovery, and restrained presentation. They are broad
inspection rules, not a tested Memoquill navigation scheme or a learning treatment.
[Author's heuristic definitions](https://www.nngroup.com/articles/ten-usability-heuristics/).

**H2 — Reversible interaction.** Shneiderman's 1983 direct-manipulation paper argues for visible objects
and incremental, reversible actions with visible results. Use this as design rationale for feedback and undo,
not as controlled evidence that dragging, animations, or scores improve learning.
[Primary design paper](https://www.cs.umd.edu/~ben/papers/Shneiderman1983Direct.pdf).

**S1 — Accessibility baseline.** WCAG 2.2 is the current published W3C Recommendation checked here
(12 December 2024). Target applicable A and AA criteria for authored web content and complete workflows.
For non-web desktop software, use the current WCAG2ICT Group Note (11 December 2025) to interpret the
criteria; it is informative guidance, not a standard to which a product can claim conformance.
An accessibility claim needs a defined product/version, scope, supported technologies, and evaluation evidence.
[WCAG 2.2](https://www.w3.org/TR/WCAG22/), [WCAG2ICT](https://www.w3.org/TR/wcag2ict-22/).

## Proposed decisions, evidence, and evaluation criteria

### UX-01 — Clear hierarchy and a recognizable next action

- **Decision:** Give each work area a descriptive title, current collection/item context, and a clearly
  identifiable primary action. Use text labels for consequential actions; do not communicate them by icons alone.
- **Basis:** H1; this exact information architecture is P, not an empirical result.
- **Gate:** In a first-use task, at least 8/10 participants identify the current location and next action
  without moderator instruction; investigate each misleading label or accidental destructive action.

### UX-02 — Navigation by recognition, with continuity

- **Decision:** Provide stable routes to collections, source material, practice, and history; retain a visible
  route back. Preserve selection, filters, and reading position when opening and returning from an item.
- **Basis:** H1 recognition/consistency; E4 motivates testing resumption, with the inference noted above.
- **Gate:** At least 9/10 participants find a previously seen item and return to their interrupted task
  without re-entering its query. Keyboard users can reach equivalent destinations without a pointer.

### UX-03 — Progressive disclosure without hiding essential state

- **Decision:** Present the common task first; disclose advanced scheduling, metadata, and diagnostics on
  request. Keep save status, selected scope, errors, and consequences visible before confirmation.
- **Basis:** H1; the appropriate grouping and density are P and must be tested with novice and experienced users.
- **Gate:** At least 8/10 novices create usable material with default settings; at least 8/10 experienced
  participants find a requested advanced setting without help. No hidden option may silently change task scope.

### UX-04 — Keep application overhead separate from learning difficulty

- **Decision:** Separate prompt, optional hint, response, feedback, and source reference. Allow deliberate
  answer reveal and revisiting explanations; do not interleave unrelated notifications with a learning task.
- **Basis:** E1 supports testing manageable presentation; H1 supports restraint. This sequence itself is P.
- **Gate:** In observation, participants distinguish their own answer from a model answer and a source quote.
  Compare interface errors and perceived effort separately from correctness, delayed recall, and transfer.

### UX-05 — Adjustable, readable typography

- **Decision:** Use selectable text with clear heading levels; offer a comfortable reading width and text
  resizing. Keep code indentation and mathematical structure meaningful; allow local scrolling where necessary.
- **Basis:** E2 and S1; any initial font, line length, or density is P rather than a universal optimum.
- **Gate:** No clipping or lost controls at 200% text enlargement. Reading content reflows at the WCAG
  equivalent of 320 CSS pixels; essential two-dimensional content has a bounded exception, not the whole screen.
  Test prose, long titles, formulas, code, tables, and mixed scripts.
  [Official reflow interpretation](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html).

### UX-06 — Light, dark, and system preference

- **Decision:** Offer light and dark appearances and a system-following option; retain an explicit override
  offline. Do not describe either appearance as universally healthier, easier, or better for learning.
- **Basis:** E3 establishes a limited polarity result; preference and environmental fit remain P.
- **Gate:** Both appearances pass contrast/focus checks, including selections, disabled states, charts, code,
  and imported content framing. Theme changes preserve drafts and location. Evaluate comfort and errors separately.

### UX-07 — Contrast, focus, and input access

- **Decision:** Meet the accessibility checks below in every supported appearance. Provide an obvious focus
  indicator and logical focus order; keep focus visible through dialogs, sticky regions, and notifications.
- **Basis:** S1; keeping the entire focused control unobscured is a stronger proposed policy than AA's minimum.
- **Gate:** Every core workflow passes keyboard-only operation with no trap or lost focus; every interactive
  control has a programmatic name/role/state and an equivalent non-drag action when dragging is offered.
  [Keyboard criterion and exceptions](https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html).

### UX-08 — Consistent language and subject independence

- **Decision:** Use consistent nouns for source, collection, activity, attempt, and evidence. Choose activity
  type by learning purpose, rather than assuming all material is vocabulary or all assessment is flashcards.
- **Basis:** H1 plus the product boundary; the precise vocabulary is P.
- **Gate:** Validate recall, explanation, problem solving, and practical evidence across four examples:
  an English phrase, a Python debugging task, an unfamiliar mathematics problem, and a cooking procedure.
  A correct recall answer must not automatically label practical skill or conceptual understanding as achieved.

### UX-09 — Recoverable editing and consequential actions

- **Decision:** Make ordinary edits and deletions recoverable; preview bulk changes with their affected scope.
  Preserve original sources and learning history. Explain errors with a cause, retained state, and next step.
- **Basis:** H1/H2; persistence mechanisms and retention windows require an architecture decision.
- **Gate:** Undo restores the prior visible content and relevant associations in scripted single/bulk cases.
  Cancel changes nothing. Recover a draft after interruption; simulate failure at each persistence boundary.
  No success message appears before durable completion, and content edits never silently erase attempts.

### UX-10 — Resume and pace a session deliberately

- **Decision:** Allow pause, stop, and resume without a time penalty. Show the last activity and an explicit
  continuation choice. Offer a small proposed workload that can be adjusted or declined; avoid compulsory streaks.
- **Basis:** E1/E4 motivate evaluation; workload defaults, reminders, and checkpoint design are P.
- **Gate:** A paused explanation, unanswered prompt, and authoring draft resume at the saved point after restart.
  At least 9/10 participants resume an interrupted task without moderator help. No timer discards user work.

### UX-11 — Honest progress and uncertainty

- **Decision:** Distinguish observed attempts, self-ratings, scheduled reviews, and estimated future recall.
  Show the date, scope, and missing evidence behind summaries; let users inspect source attempts and correct errors.
- **Basis:** H1 status visibility; this presentation is P, not evidence that dashboards improve learning.
- **Gate:** At least 8/10 participants explain what a displayed estimate means and identify what it cannot prove.
  Empty/sparse history says so. Charts have text equivalents; color is never the sole carrier of progress state.

### UX-12 — Bounded customization and large collections

- **Decision:** Keep sensible defaults; expose text scale, appearance, motion, shortcuts, and session preference
  without a setup questionnaire. Provide search, explicit filters, sorting, clear result counts, and scope reset.
- **Basis:** H1; customization scope, search layout, and performance budgets are P.
- **Gate:** Preferences persist across restart and can be reset without deleting content/history. With a declared
  10,000-item test collection, users can locate an exact title and a topic through alternate paths. Initial target:
  local search results p95 ≤500 ms after input settles on a documented reference machine; measure cold/warm states.
  Virtualized lists must expose position/count and preserve screen-reader and keyboard access to all results.

## Accessibility acceptance checklist

These checks supplement the complete applicable A/AA review; they are not a conformance certificate.
Use the normative success criterion when its informative Understanding page is linked below.

| Area | Required check and interpretation |
| --- | --- |
| Text contrast | Normal text ≥4.5:1; qualifying large text ≥3:1. Test actual foreground/background states, without rounding a failure up. [SC 1.4.3](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html). |
| Non-text information | Essential component boundaries/states and graphical information meet applicable 3:1 contrast; add labels/patterns to color coding. [SC 1.4.11](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html). |
| Focus | Visible indicator; sensible order; no keyboard trap. AA SC 2.4.11 requires the focused component not be entirely hidden by author-created content. Prefer fully unobscured controls. [Focus interpretation](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html). |
| Pointer | Targets ≥24×24 CSS px or satisfy a documented SC 2.5.8 exception, including its spacing rule. Prefer larger primary targets; provide alternatives to dragging. [Target interpretation](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html). |
| Text adaptation | At the same time, tolerate line height 1.5×, paragraph spacing 2×, letter spacing 0.12×, word spacing 0.16× font size without loss; applicable script exceptions remain. These are override tests, not prescribed default typography. [SC 1.4.12](https://www.w3.org/WAI/WCAG22/Understanding/text-spacing.html). |
| Semantics | Headings, labels, reading order, names/roles/states, language, errors, and status changes remain available to assistive technology. Test changing results and saves without stealing focus. [Status messages](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html). |
| Media and motion | Supply applicable text alternatives, captions/transcripts, and necessary descriptions. Permit stopping nonessential motion; no compulsory autoplay. Do not make timing or animation necessary for core study. |
| Shortcuts and dialogs | Discoverable commands; character-only shortcuts can be disabled/remapped or active only on focus. Dialogs announce their purpose, allow appropriate cancellation, and restore meaningful focus. [Character shortcuts](https://www.w3.org/WAI/WCAG22/Understanding/character-key-shortcuts.html). |
| Content | Imported/author-created material can be inaccessible. Preserve originals, expose missing alternatives, and provide accessible authoring/export paths; do not claim every imported PDF or formula is accessible. [Non-text alternatives](https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html). |

## Linux desktop and embedded-web caveats

Standard GTK controls expose accessibility information, but developers still supply application-specific
labels, relations, and state. Custom widgets require their own accessible implementation. This is toolkit
capability, not proof of accessibility for the finished application.
[Current GTK accessibility documentation](https://gnome.pages.gitlab.gnome.org/gtk/gtk4/section-accessibility.html).

If a WebKitGTK shell is selected, DOM semantics must reach the Linux assistive-technology tree across
process boundaries. GTK's official 4.14 account describes AT-SPI tree bridging and build requirements;
do not assume those details or that version apply unchanged to a GTK3 or different webview embedding.
A browser-only audit cannot qualify native dialogs, shell focus transitions, or packaged runtime behavior.
[GTK engineering account](https://blog.gtk.org/2024/03/08/accessibility-improvements-in-gtk-4-14/).

Before selecting a release baseline, record distribution, desktop/session, toolkit/webview versions,
packaging format, and Orca/AT-SPI versions. Test the actual package offline on the oldest supported runtime
and a current supported runtime; include GNOME and KDE, Wayland and X11 only where support is claimed.
Check fractional scaling, high contrast, reduced motion, input methods, clipboard, file dialogs, and font fallback.
Screen-reader mathematics, rich text, code editors, and virtualized collections need explicit end-to-end tests;
provide a meaningful textual route where a custom presentation is not reliably exposed.

## Qualification plan before a usability/accessibility claim

1. **Inspect early designs:** review H1/H2 and a complete A/AA applicability ledger; state why each exception
   applies. Test the hierarchy and labels before committing to visual polish. No prototype is supplied here.
2. **Formative tasks:** recruit a varied initial cohort, including keyboard and screen-reader users, low-vision
   users, and people with reading/access needs; ask for accommodations. Start with 6–10 participants across
   iterative rounds to find problems, not to estimate population prevalence or prove accessibility completeness.
3. **Representative workflow:** import local material, inspect provenance, author/revise an activity, find it,
   complete a response and inspect feedback, view evidence behind progress, pause/restart/resume, undo a bulk
   mistake, export, and restore. Run without network, AI, accounts, or cloud dependencies.
4. **Task evidence:** record completion without help, failures, wrong-scope actions, moderator assistance,
   recovery, time distribution, and post-task ease; keep participant preference separate from task performance.
   Use consented, minimized notes; no mandatory telemetry or sensitive learning-content uploads.
5. **Provisional release gates:** ≥90% independent completion across core tasks in a larger declared evaluation;
   ≥80% per task; zero unrecoverable data loss, blocking keyboard traps, or unavailable core screen-reader paths.
   Do not hide failures in aggregate percentages; report task and access-mode denominators and uncertainty.
6. **Manual access checks:** complete every core task by keyboard and with Orca in each claimed runtime;
   inspect the accessibility tree, labels, announcements, selection, and focus after every modal/state transition.
   Automated markup/contrast checks help locate defects but cannot replace these tests or disabled-user feedback.
7. **Regression and recovery:** replay drafts, undo, restart, changed sources, empty/sparse progress, long names,
   mixed subjects/scripts, and large collections. Record actual build/package and persistent-data outcomes.
8. **Learning claims:** if a design is claimed to improve learning, separately preregister a comparison with
   delayed retention, explanation/transfer, and relevant practical evidence. Usability completion is not that test.

The 8/10 and 9/10 design probes above are diagnostic targets for individual rounds, not confidence bounds.
Revise budgets and thresholds with baseline measurements before making a release decision.

## Remaining decisions and limits

- Validate the navigation vocabulary and collection model with users; no empirical source selects them here.
- Compare comfortable reading defaults at different scales, languages, lighting, and access needs.
- Qualify the chosen Linux toolkit/webview and packaging; no runtime, screen reader, or usability test was run.
- Determine undo retention and crash-recovery guarantees with the persistence design before promising them.
- Test progress comprehension without implying that a recall estimate certifies understanding or practical skill.
- This research supplies requirements and an evaluation plan; it supplies no measured Memoquill learning effect.
