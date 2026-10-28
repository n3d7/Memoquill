# M1.2 execution ledger

Authority: the user authorised implementation and a final PR **without merge** after the preparation. Base: `1374c18decc30cb6081491707139b6910f8119fa`; branch `feat/ker-98-durable-workspace`. Seven uncommitted preparation documents are preserved and included deliberately. Scope/spec: KER-98 and [preparation](m1-2-workspace-preparation.md), with the existing architecture/data/MCP documents as constraints.

## Workflow and rulings

- ub-mode owns coding, shared RED→GREEN tests, complexity/duplication/mutation checks and this action log. Superpowers verification-before-completion and one fresh whole-branch review support that workflow. Its overlapping executor/planning/TDD loops are not run again.
- G0 ruling: the explicit new request authorises storage implementation. M1.1 physical GNOME/human listening remains unconfirmed and KER-97 stays In Review. Proceed with shell-independent storage and bounded automated desktop regression; preserve the manual gate for subsequent domain/release acceptance rather than invent a pass. Cost if wrong: UI adapter work may need revision; core services/storage remain reusable.
- No new subject/learning/import/MCP runtime, cloud, accounts, unrestricted native authority or project-license change. Keep material edits ephemeral and the picker name-only. No merge, including automatic merge.
- One feature branch in the existing workspace preserves the user's prepared files and current code graph/build environment; no second worktree or competing state is created. Final review uses a bounded, explicit subagent context as requested by the selected review skill.

## Progress

- [x] G0 — source/tool/dependency preflight and entry disposition
- [x] G1 — typed contracts, private root and retained process lock
- [x] G2 — SQLite roles, safe bootstrap, checked migrations/snapshots
- [ ] G3 — settings, revisions and operation receipts/idempotency
- [ ] G4 — asset durability/reconciliation and failure/crash tests
- [ ] G5 — bounded storage owner, thin async UI and safe feedback
- [ ] G6 — qualification, independent review and PR (no merge)

## Actions and evidence

- Read current project guidance, preparation, Git and KER-98. Selected skills/tool roles before source edits. Prepared docs remain intact; the new request supersedes only their preparation-only pause. Existing Context7/Firecrawl, graph/Serena and local checks will be reused; no unrelated infrastructure is introduced.

- Baseline `npm run check` passed (UI14/Rust11 plus optimized glib regression). Selected exact rusqlite0.41.0/rustix1.1.6 and reused existing Tokio/UUID/SHA2 pins; dev tempfile3.27.0 verified from current docs before resolving. Context7 reconfirmed backup/managed-state/ACL APIs.
- G1 RED: two tests observed concurrent owners allowed and root0755. GREEN: private dirs/no-follow lock + retained File::try_lock. Canonical ID/scale tests also observed RED before validation. G1 complete suite: 6 passed plus one explicitly ignored child helper, including real two-process contention/SIGKILL release and outside-file preservation.
- Implementation layout ruling: keep a single deep workspace module with private storage submodules and a public typed facade, rather than prematurely splitting crates/layers. Tauri is kept outside those modules. Cost if wrong: a later file/module move, without a wire/schema rewrite.

- G2 RED observed incorrect journal mode and lack of schema gate; GREEN implements closed staging DB publication, per-connection FK/trusted-schema/busy policy, WAL/FULL writer and strict checksum/application/version preflight. A test fixture initially exercised public temp-root rejection instead of schema rejection; corrected it to a genuine private workspace without loosening production checks.
- G2 suite: 8 passed, including real schema1→2 upgrade and standalone snapshot, failed SQL rollback, altered-checksum preservation, real SQLITE_FULL via max_page_count, SQL writer contention, FK rejection and JSON/FTS probes. Runtime SQLite version/source/compile options are in `.qualification/m12/g2-complete.log`; no host database or full-disk operation.
