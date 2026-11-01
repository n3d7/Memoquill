# Project license change

On 2026-11-01 the user explicitly requested Apache License 2.0 with Commons Clause before continuing KER-99. Updated the root LICENSE, README, npm root manifest/lock metadata, Cargo license-file and Tauri bundle license/resources. The reuse inventory distinguishes combined project terms, prior MIT grants and unchanged third-party notices. This is a source-available combination, not plain Apache-2.0 or OSI-approved open source.

The full Apache text was retrieved from https://www.apache.org/licenses/LICENSE-2.0.txt (SHA-256 cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30). The Commons Clause condition was extracted from https://commonsclause.com/ and only its software, license and licensor fields were filled. No third-party license text or dependency license metadata was changed.

Validation passed: JSON/TOML parsing, npm root manifest/lock agreement, structural equality of all dependency lock metadata, Tauri config schema support for licenseFile, exact embedded Apache text and Commons Clause template comparison, offline locked Cargo metadata, git diff --check. Runtime tests were not repeated for this metadata/documentation-only change. A built RPM license payload still needs verification at the M2 packaging gate.

The pre-existing user .gitignore edits and all interrupted M2 source/UI edits are preserved and excluded from this license commit. Work remains on dev; no push, PR, main merge, release or Linear completion is claimed. Continue with KER-112 storage quality, then authoring/attachments/navigation and integrated acceptance.

Used: project workflow, context-efficiency, tool-routing and reporting skills; skill-catalog selection; ub-mode retained for implementation. MCP: Serena activation, Codebase Memory metadata/search/coverage for the upcoming continuation. Tools: RTK shell, Python, curl, Git, Cargo metadata, web retrieval of official license sources. No service plugin or local model was needed for this bounded change.
