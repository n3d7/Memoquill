# Course interactive-extension contract v1

KER-130 supplies a data foundation for future interactive tools. The current
policy for generated/external extensions is **inertOnly**. No executable plugin,
simulation, package loader, transport, sandbox or marketplace is implemented.
Existing trusted lesson components keep their current renderer and behavior.

## Data and authority

`src/extensions/contracts.ts` defines ExtensionEnvelope, ExtensionManifest,
ExtensionScope and ExtensionAdmission. Scope contains courseId, workspaceId,
lessonId (Unit identity), and nodeId. The caller passes expected scope from its
bound application service; imported scope is never an authority source.

The manifest carries extension identity/version, exact state schema version,
provenance kind/source, compatible host contract versions, requested named
capabilities, and reviewed host-migration IDs/from/to versions. State is JSON
data with a separate schemaVersion and payload. Content definitions and personal
learning history remain separate. Extension output is not a learning judgment.

The initial vocabulary names course-local state read/write and lesson content
read/response write. Naming a capability neither grants it nor implements it.
There is no native/database/filesystem/network/other-course capability. Every
external extension remains inert even when its declaration is recognized.

## Admission and recovery

`admitExtension(originalJson, expectedScope)` accepts at most 65,536 UTF-8 bytes,
16 levels of JSON nesting and 4,096 values. It rejects malformed/over-budget
input and mismatched ownership before retaining or returning any payload.
The bounds are engineering defaults, not learning findings.

After scope verification, unknown/newer contracts, unsupported capabilities,
invalid manifests or incompatible state are preserved as exact bounded source
text with diagnostics and activationAllowed=false. `restoreExtension` returns
that text for explicit recovery/export; it never executes or rewrites it.
`ExtensionPlaceholder` is a passive accessible view with no action bridge.
These APIs are a contract foundation, not a newly activated import/store route.
No native persistence schema is added in this step; the existing unknown-node
preservation behavior remains the current lesson recovery route.

Migration declarations may identify deterministic application-owned transforms.
No supplied scripts, eval, dynamic imports, HTML handlers or executable migration
are accepted. Migrations are not run now. A future host must snapshot original
state, validate source/target schemas and owner scope, stage a candidate and
publish it atomically; failure retains the original inert payload. Real attempts
retain their pinned definitions and original evaluation semantics.

## Runtime decision deferred

| Candidate | What must be established before adoption |
| --- | --- |
| Web worker with typed messages | Host must restrict imports/network, bound memory/CPU and cancellation, validate every message and provide an accessible trusted view. A worker does not itself authorize application access. |
| Sandboxed frame or separate process | Define origin/process boundaries, network denial, message identity, accessibility, crash cleanup and resource budgets; no ambient native bridge. |
| WASM host with explicit imports | Choose and qualify host imports, memory/fuel limits, deterministic state, accessibility and termination/recovery; no WASI/native access by default. |

A future runtime plan needs a threat model, dependency/license review,
cross-course adversarial tests, lifecycle/recovery tests and X11 accessibility
qualification before any generated code is activated. An execution model is not
chosen by storing a manifest. New capabilities need an explicit host grant tied
to course/workspace/actor/operations and object ranges. Revocation and interrupted
work must fail closed; resume requires fresh permission under the user policy.

## Evidence and checks

Reuse the existing learning-science, activity and AI-directed-learning research.
This contract makes no new learning-effect claim. Each future tool needs accurate
domain content and evidence suited to its intended learning outcome; interaction
counts, assisted correctness and simulation output do not establish competence.

Tests cover scope denial on all four identity fields, exact source recovery,
capability/version/budget handling and a passive fallback. Existing trusted
renderer tests are retained. Software tests establish behavior only; execution
security and learning effectiveness cannot be claimed for an unimplemented runtime.
