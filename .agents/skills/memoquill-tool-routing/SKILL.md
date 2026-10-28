---
name: memoquill-tool-routing
description: Use when selecting Memoquill research integrations, MCP tools, plugins, repository search, or bounded local AI assistance, or when tool availability is uncertain. Known-tool routine work does not require a catalog scan.
---

# Memoquill Tool Routing

Adapted from Scryntic tool routing; Copyright 2026 n3d7, Apache-2.0.
See `../../NOTICE` and `../../LICENSES/Apache-2.0.txt` from this skill directory.

Choose the smallest toolset that supplies the requested evidence or action. Read only the relevant reference:

| Need | Reference |
| --- | --- |
| Code relationships, semantic/literal search, Serena, local utility model | [Local tools](references/local-tools.md) |
| Scientific/technical sources, Consensus, Firecrawl, Linear, GitHub | [Research and integrations](references/research-and-integrations.md) |
| Live schemas, pagination, cost, safe sequencing | [Operating tools](references/operating-tools.md) |

1. Retain the active goal, phase, and authorization from `AGENTS.md` and the task workflow. Availability adds no permission to write externally, install services, publish, or expand scope.
2. Discover live tool names by provider/capability and inspect matching schemas only. Known identifiers need bounded reads; unknown identifiers need focused discovery. Installed, exposed, connected, and successfully exercised are separate states.
3. Inspect required fields, pagination/limits, destination, cost, side effects, actual returned content, and errors. Batch independent reads; sequence dependent calls, approvals, and mutations.
4. Reuse verified results. A missing tool in one surface, empty search, successful dispatch, or HTTP 200 proves only its narrow result. Report limitations instead of inventing availability or completion.

The dated Scryntic machine-wide catalog and provider-specific infrastructure recipes were not carried over. They contain workstation/project assumptions; discover the current session's capabilities instead. This skill does not install or reconfigure tools.
