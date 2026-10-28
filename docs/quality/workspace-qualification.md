# M1.2 workspace qualification

Local candidate for [KER-98](https://linear.app/kerigan/issue/KER-98/m12-establish-durable-workspace-and-application-service-boundaries).
The [runtime design](../architecture/workspace-runtime.md) defines the implemented
slice; [execution ledger](../planning/m1-2-execution.md) preserves decisions and
failed/passed increments. This is technical qualification, not a public release,
physical GNOME certification or evidence of improved learning outcomes.

## Acceptance evidence

| Requirement/risk | Observed verification |
| --- | --- |
| Private single owner and durable defaults | Private paths, retained no-follow lock, real two-process contention and release after SIGKILL; restart retains workspace identity/preferences |
| SQLite connection policy | FK violations rejected; defensive/trusted-schema/DQS, WAL/FULL and busy settings checked; wrong mode/FK independently rejected; runtime SQLite3.54.1/source/options recorded |
| Safe evolution and prior-state preservation | Checksummed schema1→2 upgrade with standalone backup-API snapshot; wrong/newer schema/checksum rejected; failed SQL rolls back; actual SQLITE_FULL/busy preserve prior data; existing migrations run under WAL |
| Original bytes before references | Bounded streaming/hash, exclusive private files, no-replace publication, file/directory sync before reference/receipt commit; faults and killed children before/after commit preserve interrupted/applied distinctions |
| Recovery without destructive repair | Missing/corrupt assets, unsafe shards/entries, orphan/staged bytes, and special files detected/preserved; reads remain possible when mutations are blocked; FIFO asset/database/lock cases rejected without a hang |
| Exactly-once visible outcomes | Revision conflicts and changed-payload ID conflicts; same-ID replay after lost reply; stable v1 receipt hash; corrupt/pending receipts rejected appropriately; uncertain owner closes until explicit reopen |
| Narrow async application boundary | Bounded queue/drain, accepted writes outlive reply receivers, native save→receipt round trip; remote origin/other windows/unlisted commands/forged principal denied |
| Offline useful UI slice | Explicit preference save/retry/reload/status, focus restoration and safe errors; material remains temporary; picker stays name-only; real RPM startup/restart without network |

`npm run check` passed: **20 UI tests, 62 Rust tests**, three ignored private
helpers launched by parent tests, formatting/Clippy/type/build checks and the
optimized glib regression. Crapper maximum **9.3** (<10); Dryer found no
candidates at similarity≥0.8. Initial Rust mutations exposed real test gaps;
after boundary/recovery tests and explicit internal role APIs, follow-up
**131/131 Rust** and **38/38 UI** mutations were killed, with no surviving or
uncovered selected sites and no compile-error lines. Follow-up included previously
uncovered sites; these figures are not a claim of exhaustive fault coverage.

## Packaged native candidate

- App source build: `c45c67a2d5c6ce7e4f948917aed70c0528de495f` (runtime code from
  `6dc9afd86b40be3f52667f16f653fabf84484e54`; the merge adds the user's SECURITY.md).
- RPM: `Memoquill-0.1.0-1.x86_64.rpm`, SHA-256
  `fa3b11960d69e5578edde0d56281c204c464a15dd608eceeed6a380c1a7dc984`.
- Private installed Fedora image:
  `874f1946ca2d4aa4971722a1acfcff9a350814bfb742cbfe07d8ba6eb11a418f`;
  existing desktop base:
  `4ca37a03aad13f1358bd69bd5bf03e47005d2ef0e3bdf9d6c070072b8fe4a980`.
- X11: **42 interaction/storage +7 Orca-presentation checks**.
- Native Wayland: **43 interaction/storage +7 Orca-presentation checks**,
  including protocol evidence from the restarted process.
- Both original processes closed normally with exit0 and released the lock;
  Dark/200% survived restart while the material fixture reset. The restored
  Wayland screenshot was inspected for readable layout and visible focus.

Runs used bounded rootless containers, private desktop/profile/bus, no network
or host desktop sockets, with SELinux remaining Enforcing. Outer label/seccomp
exceptions are limited to the existing disposable GTK qualification profile.
See [reproduction](../../tests/qualification/README.md). An initial Wayland run
failed because Alt+F4 sent no close event in nested Weston. The corrected run
uses GTK's accessible native close action; no forced kill substitutes for exit0.
Review also caught screenshot environment leaking X11 into restarts; the helper
now restores it, selects the child backend explicitly and checks the child log.

## Security and review disposition

- Semgrep: 236-rule source/harness run, zero scanner errors; three matches are
  `current_exe` in test-only private subprocess helpers. They grant no production
  execution authority. The separate final changed-harness run has zero matches/errors.
- Trivy: no secret matches. Dependency/config report retains the glib Medium
  version match and three Low missing-healthcheck notices for one-shot test
  containers. No bogus health endpoint or blanket suppression was added.
- The exact glib source fix, optimized regression and user-authorized
  [alert#1 dismissal](https://github.com/n3d7/Memoquill/security/dependabot/1)
  are documented in [the desktop baseline](fedora-desktop-baseline.md).
- Cargo audit: zero vulnerability records; unmaintained `proc-macro-error`
  warning RUSTSEC-2024-0370 remains visible.
- Existing Dependency-Track5.2.0 processed a 616-component BOM for dedicated
  Memoquill/0.1.0-m1.2, UUID `b3e5bc94-8ec1-451c-b5a7-808c401e62c1`, terminal
  COMPLETED. Detailed findings access returns403 with the existing key; zero
  service counters are limited visibility and do not override Trivy evidence.
- Independent read-only review found no confirmed runtime Critical/Important
  findings; its Wayland harness finding was corrected and native evidence rerun.

Raw logs, screenshots, scanner output, mutation outcomes and receipts remain
local under `.qualification/m12/`. They contain the actual execution evidence;
source publication and remote CI/merge are verified separately through GitHub.

## Remaining limits

Physical Fedora/GNOME use and human Orca listening remain KER-97 acceptance work.
Automatic polite announcements, physical power loss and all-filesystem/hardware
guarantees are not established. Startup's full registered-asset hash audit needs
a large-collection budget before M3 public imports. Complete user export/restore,
learning content/history, importer, MCP runtime and cross-platform/sync features
remain outside this increment. Root license selection/work stays paused and no
binary release is published. The newly fetched SECURITY.md is a user-authored
template preserved unchanged, not a completed support/reporting policy.
