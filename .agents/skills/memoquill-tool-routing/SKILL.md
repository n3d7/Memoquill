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

## Additional practical routes

| Need | Read only this reference |
| --- | --- |
| Semgrep, Trivy, Dependency-Track, code metrics, Fedora desktop qualification | [Verification routes](references/verification.md) |
| MarkItDown/Docling, PDF artifacts, MCP Inspector/mcpdo | [Documents and MCP diagnostics](references/documents-and-mcp.md) |

Use the installed `skill-catalog` with `--project /home/void/Project/Memoquill` for focused skill selection; load the selected skill before execution. Keep one workflow owner per phase and use specialists only for distinct supporting work. Known-tool routine work needs no catalog scan.

The references transfer useful Scryntic operating knowledge, not its dated machine-wide registry, unrelated project identifiers or server-administration authority. Workstation paths describe the configured operator environment and may be stale on another machine. Rediscover live contracts before use. This router does not install/reconfigure services or authorize cloud uploads. Record actual successful and failed usage separately from discovered capabilities.
