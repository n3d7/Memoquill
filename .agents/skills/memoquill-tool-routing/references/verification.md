# Verification routes

Adapted from Scryntic tool recipes for Memoquill. Read the named specialist's
current skill before running its workflow. Choose checks by the approved task's
risks and acceptance criteria; do not scan merely because a scanner is present.

| Question | Route | Evidence limits |
| --- | --- | --- |
| Changed-code SAST | Installed `semgrep-local` skill; explicit changed/untracked paths or approved immutable baseline | CE is not Pro inter-file analysis, SCA or secret coverage; report rules, engine, errors and skips |
| Dependency advisories, IaC, secrets, SBOM | Installed `trivy-local` skill; select actual scanners and targets, then retrieve findings | Record vulnerability DB date and lockfile scope; an upload or empty summary is not proof of absence |
| SBOM component analysis | Existing Dependency-Track service, dedicated Memoquill project/version | Wait for BOM processing and analysis; distinguish ingestion success, advisory counts, finding access and complete review |
| Diff security review | Installed Codex Security `security-diff-scan` skill when applicable | Pin base/head or patch identity; do not substitute a repository-wide Deep scan |
| One repository/path audit | Installed Codex Security `security-scan`; Deep only for an explicitly exhaustive/multi-pass scope | Static evidence does not authorise live exploitation, finding publication or unrelated scans |
| Complexity, duplication, mutation quality | The selected coding workflow; `crapper`, `dryer`, `mutator` if required by that workflow | A metric is not a behavior test; build failures/timeouts are not killed mutants |
| UI and core correctness | Current `package.json` scripts, Cargo configuration and targeted behavior tests | Record the exact revision and failed/skipped checks separately |
| Linux desktop behavior | `tests/qualification/README.md` and current Fedora baseline in `docs/quality/` | Browser tests do not prove native WebKit, keyboard, AT-SPI, IME or Orca behavior |

## Semgrep and Trivy details

Semgrep's exposed custom-rule tool needs valid YAML starting with `rules:`.
Do not invent an ordinary scan method from a different MCP. The installed skill
also documents a CLI/helper route. A CE baseline can omit unstaged/untracked
changes; explicitly cover this task's new paths without staging user work to
make a scanner see it. Keep metrics/cloud/Pro settings within existing policy.

Trivy filesystem MCP can return a batch ID and counts rather than full findings.
Retrieve `findings_list`, paginate, then inspect material `findings_get` records.
Explicit severity selection matters; HIGH alone may omit CRITICAL. Use CLI for
SBOM output or unsupported scanner options, and reuse existing JSON with
`convert` when appropriate. DB refresh may use public network even for a local
scan. Redact secret matches; never paste a matched credential into reports.

## Dependency-Track and lab boundaries

The user's existing `scryntic-lab` host can provide shared development services;
it is separate from the Scryntic repository, which stays read-only. Use the
installed `scryntic-lab` skill only when actual host operations are authorized.
Find the exact Memoquill target and existing non-secret receipt/config pointers
in the current handoff or qualification docs. Keep API keys in their existing
private files; do not print them. Do not alter unrelated projects or register a
second project/version to evade an access failure. Record insufficient finding
permissions explicitly. Do not run the Scryntic product's monitoring wrapper
against Memoquill without inspecting its scope and destinations.

SonarQube is currently not configured for Memoquill and the user excluded it.
Do not query the Scryntic Sonar project as a substitute or install a server.

## Native qualification and completion

Use existing bounded qualification scripts and private profiles. Keep SELinux
Enforcing. Where documented host-user execution is needed for Podman/GPU/user
bus access, use the approved permission mechanism instead of weakening the
host or silently switching runtime. Record the tested X11/Wayland/runtime,
artifact checksum, automation limits and manual checks still outstanding.

Compare evidence to each acceptance criterion. Scanner findings are candidates
until triaged; a remote dispatch is not a terminal gate. After a relevant fix,
rerun the affected check and the required final gates, not every historical
scan. Keep local results, remote CI, tracker state, publication and merge proof
separate. No merge follows from permission to create a PR.
