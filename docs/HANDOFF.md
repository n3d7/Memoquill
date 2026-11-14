# Current objective — M4 merge candidate and tracker/protection follow-up

2026-11-14. User now authorizes PR #7 review/merge into main if ready, Linear
state updates and main protection after merge. Bounded Spec review found one
KER-102 gap (deliberately flawed example label/correction/review provenance).
Corrected with opt-in constructed v3 errorRepair; follow-up review found no
remaining blocker at the reviewed seams. Focused21/0/5 helpers, StudyPanel12/0,
types passed. Catalog-count/schema guard corrected after first full543/1; focused
catalog1/0/5 helpers passed. Only test code changed; shipped runtime unchanged.
Current RPM private native GUI12 assertions and actual normal
close/lock release/reopen passed, no screenshots or workstation interaction.
Review receipt: [merge gate](task-reports/2026-11-14-m4-pr-review-and-merge.md).
Final full must pass before publishing/merging the corrected candidate; source
snapshot and final receipt: .qualification/m4-merge-candidate-20261114/.
PR worktree .qualification/m4-pr-finalize; original dev dirty work preserved.
Next after successful full: commit/push only M4 correction+receipts, exact-head
merge, read back main, apply/read back main protection, then mark KER-102/131/103
Done with evidence. Until then issues are In Review. Original workstation failure
cause remains unattributed; current guarded native normal-close/reopen passed.

# Current objective — M4 pull request to main and isolated GUI qualification

2026-11-14. User authorized committing/pushing the M4 scope, opening a PR into
main, and isolated GUI interaction/screenshots. Merge and tracker writes remain
unauthorized. Existing local environment/verification-runner work is excluded
from this PR and preserved in the original workspace.

M4.1/KER-131 and M4.2 implementation is integrated: offline study, immutable
history, pinned FSRS7 recall, practical reminders and frozen scoped assessments.
Local full qualification passed 540 tests/0 failures/10 private helper skips with
audit passed. This total includes 56 local runner self-tests; the runner utility
itself is not part of this PR. Product code matches the qualified source snapshot.
Native GUI evidence is recorded in docs/task-reports/2026-11-14-m4-pr-gui.md.
No native GUI or release-level accessibility pass should be inferred from local
unit tests. KER-97 human Fedora/X11/Orca acceptance remains M7.

Next: review this PR and its actual remote checks; no automatic merge. Keep local
native RPM/image/screenshots and summary receipts until review. Main must remain
unchanged until separately authorized integration.

## Previous main handoff — preserved

# Current objective — M3 qualified, publish dev→main PR

2026-11-13 14:52 UTC. User explicitly resumed M3. Software acceptance complete;
publication still to perform/read back. NO merge/auto-merge/force/history rewrite,
NO M4. Preserve unrelated staged AGENTS/.agents removals, dirty tools/package.json/
notices and optimization reports. No agents/GUI processes running; screenshots OFF.

Code a03431a includes59b2177 final-review corrections and distinct-originals duplicate
guard correction. Full476PASS/0failed/8skip instances; isolated native full109PASS/
0failed/0skipped, both auditsPASS. No doctests declared; no Sonar/remoteCI/release
claim. Native full verifies actual original byte/name copies and G1/G2 new-course
roundtrips. Evidence `/home/void/Project/Memoquill/.qualification/verify/20261113T163954-3df20470/summary.json` and `/home/void/Project/Memoquill/.qualification/verify/20261113T164619-cb226955/summary.json`.
Report: docs/task-reports/2026-11-13-145217Z-m3-completion.md. Headless code checks remain valid after harness/docs-only edits.

Next: commit only owned harness/M3docs/report, push dev, create and read back one
English dev→main PR using prepared pr-body.md, then record publication receipt.
No open PR found on last GitHub search. Never merge or start M4 automatically.

## Historical continuation details

# Current objective — finish M3 (explicitly resumed)

Updated 2026-11-13 14:46 UTC. User resumed all M3/KER-100+101. Implementation/dev commits,
push dev and English dev→main PR authorized; NO merge/auto-merge/force/history
rewrite; NO M4. Russian replies, English artifacts. Preserve user staged AGENTS/
.agents removals, dirty tools/package.json/notices/optimization reports. GUI only
isolated synthetic X11, screenshots OFF, no personal courses. No agents active.
Use external runner, final JSON only, failure logs only for concrete errors.

HEADa03431a:59b2177 fixes both independent final-review Important findings
(ordinary selection survival + persisted/historical ingestion domain refs).
a03431a also fixes confirmed real backend bug: ordinary duplicate guard formerly
rejected all different originals on one holder; now scopes to owner+digest.
New real DB CSV→JSON→renamed-identical-CSV regression RED second.json InvalidRequest,
GREEN permits distinct inputs and rejects renamed same bytes. Both new tests and
actual rows/hash-recomputed package mutants verified. Scope reviewed by primary.
Latest full external runner PASS476/0failed/8reported skips, auditPASS, no policy
findings: .qualification/m3-final-full-v2.json, report
verify/20261113T163954-3df20470/summary.json. Code unchanged after gate except gitcommit.
No repeat of unchanged headless gates needed for harness-only edits.

Native v3 actual updated RPM now PASSED complete CSV/search/rebuild/reimport/undo
AND named JSON apply/reconciliation (previous blockers fixed). It then failed a
HARNESS race: Save dialog closed successfully, exact originals/import.csv exists,
status Saved original bytes, but click('Save') waited for absent button and recorded
false failed entry. Evidence .qualification/m3-final-native-v3.json and
verify/20261113T164203-8cd975f7/native-evidence. This is not whole M3PASS.
Parent just fixed optional Save/Open confirmation via native AT-SPI doAction without
failed wait journal entries when dialog already closed. Mandatory exact filename,
byte hashes and real inspected/imported packages remain strict; removed old .csv.csv
fallback. Python syntax/diff checks next, then native full rerun under new hypothesis.
Uncommitted harness tests/qualification/m3-journey.py; no product code change.

Next runner invocation/result: .qualification/m3-final-native-chooser.json
python3 tools/verification/verify.py run --profile native --native-image
localhost/memoquill-m1-desktop:20261027 --native-input .qualification/m3-006c1d2/input
--scenario full --hypothesis '<successful auto-closing dialog confirmation race>'.
May be in flight: inspect existing final JSON/status once before launching anything.
No agents, observers, mqwatch or manual GUI. All prior runs cleaned owned containers.

Still INCOMPLETE: final original PDF/EPUB/DOCX/HTML copies, G1/G2 full UIroundtrips,
final qualification matrices/report/HANDOFF, commit owned docs/harness, push dev,
create/readback PR; NO merge. No M3push/PR; origin/main277b8f6 only prior2merge
commits beyond f80840d (same tree); latest open dev→main search empty.
Prepared English PR body in .superpowers/sdd/2026-11-09-m3-completion/pr-body.md.
Old independent reviews/fix reports in same ignoreddir. Full report/audit successes
are distinct from skipped/manual M7/remoteCI; no doctests declared, no Sonarclaim.
User permits improving test software for concrete defects. Quota cutoff is NOT a
user pause; continue authorized M3 on capacity recovery, checkpoint-first.

## Previous runner handoff

# Current objective — dedicated verification runner

2026-11-13. User authorized runner implementation and a usage-memory note.
M3/KER-100/KER-101/M4 remain paused. No publication or product-development authority.
Branch dev; HEAD 006c1d2df3499b0cda3ca9f802706203e4f53a5a; existing dirty M3 work
and staged AGENTS/.agents removals preserved. No new commits or remote writes.

Terminal attachment: `python3 tools/verification/verify.py watch` (read-only TUI;
Q/Ctrl+C detaches without stopping tests). No web panel or server.

Default checks: `python3 tools/verification/verify.py run --profile affected`.
Use --frontend PATH / --rust NAME for a focused regression; quick for types/fmt;
full (npm run check) for all headless gates; native only for isolated desktop work.
Read final JSON, then only relevant failing evidence. No agents or LLM log parsing.
`status`, `failures`, `report --json` inspect results without executing tests.

Pinned Nextest/JUnit and isolated Hypothesis now run through the same CLI.
Optional licenses profile records policy review; --resource-scope is host-only.
Target correction: --only rust-doc --only runner-self-tests (not full/native).
Details: docs/engineering/verification.md.
Report: docs/task-reports/2026-11-13-runner-acceptance.md.
Final verification: .qualification/verify/final-validation.json.
Pre-task backups: .qualification/runner-baseline/.
Host pass reuse is disabled without verified immutable environment/dependency roots.
The save-dialog scenario passed live isolated X11 qualification with exact filename,
cancellation and byte-hash checks; no screenshots. Report schema 2 audits raw evidence.
User explicitly authorized fixing all validation issues: existing Rust formatting
and equivalent Clippy condition style corrected; no M3 feature development resumed.
Pinned license rules are guarded by existing notices/source/hash evidence.

Next: use the runner for subsequent authorized engineering. Product work remains
paused until the user explicitly resumes it. Earlier product evidence is preserved
in docs/engineering/legacy/HANDOFF-20261113.md.
