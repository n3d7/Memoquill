# Documents and MCP diagnostics

Adapted from Scryntic's local utility and task-recipe guidance. These are
workstation tools, not Memoquill runtime dependencies or installed importers.
Read actual versions/help only when needed; examples are not availability probes.

## Document routes

For a quick readable extraction from ordinary Office/PDF input:

```sh
rtk proxy markitdown /absolute/input.docx -o /tmp/document.md
rtk proxy markitdown -x pdf < /absolute/input.pdf > /tmp/document.md
```

For complex PDFs, tables or scans load the installed `docling` skill, then use
the existing local wrapper:

```sh
rtk proxy docling-local /absolute/input.pdf --to md --output /tmp/docling-result
```

Use `--to json` for structured output and `--ocr-engine rapidocr` when OCR is
needed and supported. The configured wrapper uses CPU, preloaded models at
`/home/void/.local/share/docling/models`, offline Hugging Face lookup, disabled
telemetry and remote services. This is documented configuration, not an OS
network sandbox or a current health result. Preserve inputs and keep incidental
conversion outputs in a task-owned `/tmp` directory. Do not update tool/model
environments during unrelated work.

Check conversion status, page count, text and material table cells. Extraction
can lose equations, layout or content. For PDF/DOCX creation or visual fidelity,
use the installed `pdf`/`documents` skill and its rendering checks; Markdown/OCR
alone cannot establish visual correctness. Other artifact skills are selected
through `skill-catalog` only for an actual spreadsheet/presentation task.
Hosted parsing is an upload requiring the appropriate data authorization.

## MCP Inspector and mcpdo

Use the installed `mcpdo` skill for inspecting/testing existing MCP connections
instead of writing an ad hoc MCP client. Its daemon/server/connection inventory
describes that CLI, not the full built-in app/MCP registry. An empty Inspector
catalog does not establish that a built-in integration is absent.

For an authorized finite server diagnostic, the existing Inspector CLI supports:

```sh
rtk proxy mcp-inspector --cli <server-command> <server-args> --method tools/list --strict --format json
```

Tool invocation uses `--method tools/call --tool-name <name>` and
`--tool-args-json` with the actual advertised arguments. Starting a server or
calling its tools can have effects; a diagnostic request is not general write
authority. Keep credentials out of arguments/output. `MCP_INSPECTOR_SECRET_STORE=memory`
is for transient CLI diagnostics, not persistent multi-process `mcpdo`.

Adding, installing or configuring a new MCP requires the installed
`mcp-precheck` workflow and task authority. Distinguish PolicyLayer registry
checks, ToolCheck, source inspection and actual connection success. A registry
record is not a security guarantee; an unknown record is uncertainty. Inspect
external identifier/data destinations before calls. Do not route around an
approval rejection via another provider or transport.
