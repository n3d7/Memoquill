# M2 native constructor acceptance and original-preview focus

Completed the remaining constructor checks on the actual Fedora/X11 host in a
separate synthetic profile. M2 implementation/X11 acceptance advanced; final
platform acceptance remains incomplete under [KER-116](https://linear.app/kerigan/issue/KER-116/m28-qualify-integrated-m2-integrity-security-and-fedora-accessibility).
The user explicitly deferred M1.1 desktop setup. No host packages were installed.

- Exercised recall, single choice, constructed response and procedure through
  native keyboard/AT-SPI controls. Correct/wrong exact choices differ; authored
  rubrics, two steps and reflection guidance survive save/restart.
- Verified independent lesson prompts, responses, hints, checklist state and
  reflection across tab switches. Preview responses reset after process restart;
  these simulations do not become persisted study history.
- Exercised stack/columns/section/material/original/exercise/button/status;
  canonical text and provenance render, a named action changes only its section,
  and a one-hop local status follows that section. Earlier English content keeps
  its original section without the newer exercise; closing it preserves current content.
- Original HTML appears as literal source, including script/button markup.
  Found a real keyboard defect: closing a lesson original removed the focused
  Close button without restoring focus. LessonComposer now retains its own opener
  and restores it, with a visible lesson-heading fallback; hidden lessons cannot
  receive restored focus. No service/storage/schema/permission change.
- Regression first failed with focus on body; then11 targeted tests passed.
  Final affected lesson suite:47 tests/8 files passed. TypeScript/Vite and RPM
  build passed. The rebuilt RPM payload passed the original native focus case
  and post-restart exercise/component checks. Reused unchanged broad Rust,
  security and integration-review evidence from the prior checkpoint.

RPM SHA256: d2c55cf523f97c5728ce63c9d673f98526ceb7f2e5bd2326236949ff5a12b9db.
Payload SHA256:92639334369726e9cc9ae9dcdbe3394cd306d040bd5c713e0531ce7e36203ec8.
Candidate and source hashes: .qualification/m2-composition/focus-fix-candidate/candidate.json;
assertions: host-journey.json; affected tests: focus-fix-tests.log in that workspace.
Some revisited select options were absent from AT-SPI; Space/keyboard navigation
plus rendered/durable outcomes confirmed the actual choices. Harness assumptions
about source prerequisites and flattened snapshot ancestors were corrected; their
failed attempts were not counted as product passes. On resume the old process was
absent; its stale window was rejected before input and the same candidate relaunched.

Still open: saved nested-Wayland abort, failed Orca presentation audit, physical
GNOME/human speech and full visual/reflow acceptance. No further cold-Orca retry,
new research or screenshot capture. See [qualification](../quality/m2-qualification.md)
and the [deferred M1.1 plan](../planning/m1-1-final-acceptance.md).

Owned changes: LessonComposer and its tests, qualification/continuation records,
and the M1.1 deferral plan. Preserve unrelated AGENTS.md, .gitignore, project-skill
edits and the two November1 tooling reports. Base e84dca6 on dev; local substep
commit only, no push/merge/PR/release. KER-120 stays In Review; KER-116/99 stay In Progress.
Next: resume remaining platform qualification when its deferred prerequisites can be met.

Used: Superpowers debugging/TDD/verification/inline execution; project workflow,
context and reporting skills; Codebase Memory and Serena MCP; Linear plugin;
RTK/Git, Python/AT-SPI/xdotool/i3, npm/Vitest/Tauri/RPM. Headroom recovered one
bounded stale-window error. No web/research plugin or new security scan in this slice.
