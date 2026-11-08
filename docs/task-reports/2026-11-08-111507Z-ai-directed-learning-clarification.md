# AI-directed learning clarification

Recorded 2026-11-08 UTC. Complete bounded evidence/requirements checkpoint;
design decisions remain open. User prioritizes learning productivity and broad
AI/MCP direction/assessment; this is explicitly provisional and research-led.

- Reused learning model, activity evidence and MCP autonomy contracts; no repeat
  collection of known tutor studies. Current architecture already supports broad
  delegated authoring, but not unrestricted mutation of attempts/grades.
- Checked one new grading study through Consensus and its primary publisher via
  Firecrawl Cloud. Native arXiv exact-title search returned0. Fetch URL-as-ID
  failed; corrected hash fetch succeeded. No repeated failed request.
- [Evidence brief](../research/ai-directed-learning-brief.md) records methods,
  applicability and limits: grading agreement differs from learning effectiveness;
  model consistency is not proof of correct assessment.
- No product code, dependencies, GUI, screenshots or roadmap/Linear changes.
  No tests/scans needed for this research-only checkpoint. Local documentation
  links/diff and preservation of five unrelated user files are checked.
- M2 PR #3 remains separate published work; this research checkpoint stays local
  on dev and is not pushed into that PR. Owner retains merge responsibility.

Open: AI cost/latency constraints, host availability and uncertain-assessment
policy. Next: ask the user about practical resource constraints before designing.

Actual use: Superpowers brainstorming; project learning-evidence/workflow/tool
routing/reporting; Firecrawl skill. MCP/plugins: Consensus search/fetch, native
arXiv search, Firecrawl Cloud balance/search/scrape/feedback. Tools: RTK/Python/Git.
No new global-memory read, web-tool fallback, implementation or publication.
