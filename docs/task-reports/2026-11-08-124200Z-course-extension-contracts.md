# Inert interactive-extension foundation

[KER-130](https://linear.app/kerigan/issue/KER-130) data-contract foundation is
implemented. Native persistence, executable runtime and import activation remain
deferred as designed. The concurrent KER-129 integration is still in progress.

Added src/extensions contracts/admission, passive accessible placeholder and
tests; docs/architecture/course-extensions.md specifies ownership, capabilities,
versions, compatibility, host-owned migrations, recovery and runtime alternatives.
Foreign course/workspace/lesson/node scopes fail before returning payloads.
Bounded matching-scope JSON retains its exact source even for unknown versions
or incompatible manifests. Capabilities are declarations, never authority.

Checks: initial missing-API RED; 13 extension/trusted-renderer tests GREEN.
Independent review found object-to-string coercion in provenance.kind. Two
regressions reproduced the exception/invalid-array acceptance (RED), then strict
primitive validation fixed both: final focused result 15 passed across 2 files.
Reviewer confirmed spec and quality; minor deferred coverage: separate v1 state
mismatch and migration-declaration branches lack dedicated tests. No execution,
security sandbox qualification or learning-effect claim follows from these tests.
Full build temporarily encountered the deliberately missing Task2 port during
that worker's RED phase; final integrated build remains Task2's gate.

Branch dev; local-only substep commit follows review. No push/merge/GUI/CI or
tracker write. Unrelated AGENTS/.gitignore/workflow-skill/old-report work preserved.

Used: Superpowers TDD/verification/review, engineering-workflow, Memoquill context
and workflow/reporting skills; Codebase Memory and Headroom MCP; RTK, Git, npm,
Vitest, apply_patch. No new dependencies, external research or runtime plugin.
