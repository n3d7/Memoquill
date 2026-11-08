# Memoquill continuation checkpoint

Post-M2 discussion, 2026-11-08: user confirms each top-level tab/sidebar entry is
an entire isolated course with its lessons/tasks/materials/UI/history/agent
context inside. The learner sets goals/focus; any external model through MCP may
progressively author/adapt the course. Provider/tariff/budget is not a prerequisite.
Separate post-M2 step [KER-127](planning/course-workspace-followup.md), Backlog;
requirements recorded, architecture/migration design and implementation not begun.
Full visual design remains deferred. Post-M2 research/planning stays local only.

Updated 2026-11-08 11:27 UTC. Branch dev; post-M2 planning commits local only.

- M2 functional X11 work is complete. User explicitly authorized publishing dev and creating PR; earlier no-push/no-PR instructions are superseded. PR #3 is merged by the owner: https://github.com/n3d7/Memoquill/pull/3. No agent merge, release or M3.
- User requires every M2 substep commit to remain visible in main after eventual integration, with original SHAs. Use fast-forward or an ordinary history-preserving merge; never squash/rebase or replace the history with one commit. Owner will perform the integration; do not merge or enable auto-merge.
- Linear readback: KER-99 and all12 M2 children (109–120) Done. KER-97 remains In Review/unaccepted. Historical tracker pauses are superseded by the explicit resume.
- Main M1–M8 is X11-only. Portable versioned models, typed frontend/controller/service/native adapters and capabilities remain required. Standalone later-platform tasks stay inactive after M8 with no core dependencies; no SDK/port/runtime work now.
- Different editable lessons remain fully isolated: content/forms/answers/hints/progress do not propagate. No linked-tab editing or future linked-tab seam.
- Full visual-design timing will be discussed with the user after M2. Do not add a redesign or alter the UI roadmap before that discussion.
- Final exit fix: labelled text inputs/textarea use block layout to avoid a valid WebKitGTK AT-SPI inline-label query crash confirmed on X11. Names/implicit labels are retained; checkbox/radio/hidden controls excluded. This is an app layout workaround, not an upstream engine fix.
- Same updated probe fails on the old RPM and passes on the candidate. A first candidate probe rejected a valid separate label representation; corrected harness still reproduces the old failure.
- Final frontend coverage120/23 passed; targeted preferences6 passed after waiting for the unchanged125% root-font effect. TypeScript/Vite/Rust release/RPM build passed.
- Fresh isolated packaged X11 journey passed68 assertions plus one IME metadata record, exit0: authoring, code/math/real IME, originals/search, constructor families/components, isolated lessons and save/restart. Restart uses the same candidate through PATH. Zero captures.
- Final RPM SHA256 5c0adfe50a553e15675cb87ecbd7e5525c691d3a176973e269fd984c1d96a394. Candidate/source hashes and logs: .qualification/m2-x11-exit-20261107/.
- Unchanged157 Rust-library tests/3 ignored,4 startup tests, fmt/Clippy, optimized glib and prior bounded security evidence reused per user cadence. Source/locks unchanged except the CSS and test/harness exit delta.
- Prior whole-change review findings were fixed in3b1c5a4/1c1fb43. One fresh independent review of the source tail after1c1fb43 found no Critical/Important or confirmed Minor issues.
- Full X11 release AT/reflow/human speech remains M7. UI-only mode is X11+M2 guarded and explicitly records speech NOT_RUN. Earlier Wayland/Orca failures and cold-readiness NOT_RUN remain retained, not passed. No GNOME/Orca host install or host GUI control in this exit run.
- Catalog refresh:1362 base/1367 project entries, zero stale/missing metadata errors. Health/auth unprobed. Jupyter/Chrome DevTools/public Grep/DeepWiki/offline n8n were discovered, not exercised.
- Preserved unrelated workflow skill, .gitignore, AGENTS.md and both November1 tooling reports; all five hashes match the saved baseline. Never stage or revert them.
- Exit source commit8c9c27a. Refreshed origin/main dcac149 contains the owner merge; all M2 commits through50fece5 remain reachable. Local dev has additional unpushed research/planning; no local merge/checkout. [Main-readiness record](task-reports/2026-11-08-105831Z-m2-main-readiness.md).
- Current report: [M2 X11 exit](task-reports/2026-11-08-105345Z-m2-x11-exit.md); [qualification ledger](quality/m2-qualification.md); [execution ledger](planning/m2-execution.md).
- GUI permission exists if truly needed, but agents never capture screenshots. After reboot verify any process/window anew; old host IDs are not evidence.

Next: continue course-workspace requirements, then agree a design for KER-127. Preserve user files and dev; no agent merge or automatic M3. Full visual-design timing remains for discussion. [Publication record](task-reports/2026-11-08-110548Z-m2-pr-publication.md).
