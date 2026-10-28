# Research and integrations

## Evidence

For version-sensitive library/API questions, use Context7: resolve the intended
library, then query the exact `/org/project[/version]` ID for one uncertainty.
Inspect live per-question limits and reuse valid IDs/results. Pin conclusions
to the installed version/source and focused tests where behavior matters.
Context7 receives the query externally; use public API names, not private code,
credentials or learning data. For Memoquill, common questions concern Tauri
managed state/capabilities, rusqlite connection/backup behavior and React UI
contracts; finding an example does not establish its fit for this version.

Discover the connected Consensus tools for focused literature questions. Respect the user's remaining quota. At the start of the initial Foundation assignment the allowance was two queries; consult the current handoff for consumption rather than resetting it on continuation. Independently verify important papers and bibliographic details.

Prefer primary papers, systematic reviews/meta-analyses, official specifications, and actual repositories. Record design, outcomes, limitations, applicability, and product consequences. A model-generated summary or popularity is not scientific evidence.

Prefer the existing self-hosted Firecrawl integration for public search/scraping when available; discover its actual schema. Use hosted retrieval or another web tool for missing evidence when appropriate. Distinguish local accounting from paid credits and respect budgets. Do not transfer Scryntic lab/admin procedures.

Read supplied URLs rather than inferring contents. Check versions/dates for mutable technical facts and preserve direct source links. Never send private project or learning data to a provider without appropriate authority.

## Linear and repository services

Before creating records, identify the workspace/team, find any existing Memoquill project, and inspect relevant issues for duplicates. Never attach work to Scryntic, Chronlyt, or unrelated projects.

The initial Foundation assignment authorizes its dedicated project/backlog creation. Existing authorization carries forward; do not repeatedly request it. Resolve consequential workspace/team ambiguity through connected context or clarification instead of guessing.

Keep `docs/planning/roadmap.md` as the implementation sequence and `docs/planning/linear-backlog.md` as the mapping to successfully created IDs/links. Inspect actual results before claiming writes. On failure, preserve local definitions and the precise blocker.

Git, GitHub, Linear, and CI each prove different facts. Do not commit/push/publish or modify unrelated issues without authority. Inspect actual licenses and per-file notices before reuse; visibility alone grants no rights. These development integrations are not dependencies of core Memoquill operation.

## Firecrawl operation and fallback

Prefer the existing local `firecrawl_selfhost` MCP for public search, scrape,
mapping and bounded crawl. Inspect `success`, actual retrieved content and
crawl terminal state. Use small search/map limits (for example 3) and a bounded
crawl (for example 1 page). Health alone is not retrieval; an empty search/map
alone does not establish service failure. Cloud-style schema options do not
prove the self-hosted server implements them. In this workspace, scrape has
previously rejected the advertised `toolDetail` field; omit it rather than
repeat the known invalid request.

If the MCP transport fails, the already installed SSH-backed CLI can retrieve
the needed evidence without adding a tunnel or service:

```sh
rtk proxy /home/void/.local/bin/firecrawl-selfhost health
rtk proxy /home/void/.local/bin/firecrawl-selfhost search 'SQLite backup API official documentation' --limit 3
rtk proxy /home/void/.local/bin/firecrawl-selfhost scrape 'https://www.sqlite.org/backup.html'
```

It also supports `map URL --limit 3`, `crawl URL --limit 1` and
`crawl-status ID`. Preserve the existing private Debian loopback binding
`127.0.0.1:3002`, SSH host checks and disabled agent forwarding; the service is
not bound on the workstation. An unavailable server does not authorize restart,
reprovisioning or policy changes. Server operations require the installed
`scryntic-lab` skill and the current task authority; they do not permit writes
to the Scryntic repository.

Use hosted Firecrawl or built-in web retrieval for missing evidence when the
local route fails or an actual hosted capability helps. Local and hosted
contracts differ; inspect required fields such as the hosted `objective` rather
than copying arguments between them. Check applicable credits before paid jobs,
bound the request and stop paid retries on insufficient-credit/402. Local
`creditsUsed` accounting does not establish cloud billing. Do not enable new
providers, anti-bot services or TLS bypass for routine retrieval.

## Papers and asynchronous research

Consensus search/fetch should preserve the exact paper record before citation.
For an authorized deep review submit one complete brief and poll the returned
thread according to its contract (historically at least 30 seconds apart).
Running/dispatch states are nonterminal; save only a verified final result.
Deep mode, library writes and quota consumption retain their actual authority.
The supplied Consensus PDF is existing evidence, not a fresh quota allowance.

When the arXiv integration is exposed, search narrowly and inspect the paper's
identifier, revision, date and publication status. Distinguish a preprint from
peer-reviewed or replicated evidence. Verify important claims against the
paper and independent support where available. Use public primary technical
specifications for engineering questions. Update `docs/research/` with findings,
limitations and product consequences; do not re-open broad literature reviews
for a routine implementation decision already settled by source/tests.

For authorized GitHub delivery, validate the intended immutable base/head,
create the body through a structured argument or a temporary `--body-file`,
and verify the returned PR/head/state. Empty or absent CI is not passing CI.
Keep merge/auto-merge disabled when only PR creation was authorized.
