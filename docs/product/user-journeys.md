# Core user journeys

These are acceptance scenarios, not reports of user testing. Each must have an offline path. Common requirements are defined in [product-spec.md](product-spec.md); technical gates appear in [verification](../quality/verification-strategy.md).

## A — Start a completely new subject

From the start view, create “Quantum physics” without selecting a template; optionally describe prior knowledge and a goal. Add/relabel collections, place an explanation in more than one relevant location, link complex-number prerequisites, and make a personal path. Create a problem with a worked solution, then move its topic.

**Success:** meaningful structure exists without a predefined curriculum or subject-specific form; moving changes no IDs, citations, attempts, or prerequisite meaning. Cancel/undo a move and recover from a rejected prerequisite cycle with a clear explanation.

## B — Learn from an imported dataset

Create English; choose a small properly licensed CSV with source IDs, lexical senses, definitions, and authored examples. Review encoding/column mapping, attribution, duplicates, and proposed recall activities before applying. Open original rows and edit a derived lesson. Study recall plus sentence production and revisit mistakes.

**Success:** source bytes/hash/license remain accessible; accepted/rejected counts reconcile; recognition/recall/production are labeled separately; reimporting unchanged data creates no duplicate activities or lost progress. Missing mandatory attribution prevents redistributed example-package approval, not private access to the learner's original file.

## C — Turn imported information into lessons using AI

Import Markdown, notes, and PDFs. Preserve every original; display extraction status/limits. In Connections, authorize one external agent for selected subjects/resources and proposal submission; progress access is off by default. The agent discovers/searches material and submits a typed plan: units, prerequisites, explanations, exercises, source anchors, and gaps.

In default mode, review changes/citations in the application, edit/reject parts, and apply the exact validated selection. Alternatively grant bounded reversible authoring in a selected area: the agent can iteratively create/organize lessons and exercises without a click per safe operation, inspect previews/validation, resume its persistent draft after interruption, and apply only permitted operations. A sensitive change or concurrent human edit yields review/conflict rather than silent overwrite. Disconnect/revoke afterwards.

**Success:** an external agent can build and revise a substantial multi-module course using the shared model, citations, prerequisites and several activity types, inspect the result, and resume work. There is no access outside the grant, direct DB manipulation, arbitrary file/URL import, permission self-grant or silent source replacement. Default review and explicit autonomy are tested separately; sensitive changes require review. AI authorship/unverified state remain visible. Unsupported PDF extraction is explicit; originals can be opened/disclosed as bounded resources. The host's subsequent use of read data is outside Memoquill's enforcement boundary and explained before access.

## D — Continue without AI

Disconnect every integration, disable network, open an existing collection, author a lesson/activity, search sources, complete/resume practice, review due items, and inspect progress. Export and back up.

**Success:** all core workflows function without credentials, queued cloud jobs, AI-generated answers, or service timeouts. Optional integration controls are secondary and never block study.

## E — Practical learning

Create Cooking; add a complex recipe, link component techniques, and attach a diagram with alt text. Add recall of relevant facts separately from a procedure with ordered steps, checkpoints, observations, and a self-assessment rubric. Complete the recipe in the real world and record what happened.

**Success:** the application differentiates “recalled”, “attempted”, and “self-reported procedure completed”. It never infers verified competence or food safety from quiz accuracy. A skipped step remains distinguishable from a successfully completed step.

## F — Extensive personalization

Choose system/light/dark appearance, resize reading text, modify a non-reserved shortcut, save a filtered Library view, set subject overrides, reorder collections, and adjust session activity/workload preferences. Reset one category.

**Success:** default and customized workflows remain keyboard-operable; shortcut conflicts explain recovery; contrast/focus cannot silently disappear; reset does not alter content/history. A user can always reach Settings and a reset action without a pointer.

## G — Recovery and portability

With meaningful activity/history across four subjects, test two separate paths with network disabled and a fresh supported application instance. Verify manifests, asset checksums, versions and counts before activation.

**G1 — Native full-backup restore:** restore into a new workspace before switching. Stable IDs, relationships, revisions, all source snapshots/attribution, annotations, attempts, schedule generations/due dates, drafts/resume positions and preferences agree with the backup. Externally linked originals must have captured snapshots for a complete backup; omitted/missing files are enumerated and the result is labeled incomplete. Reauthorize agents rather than restoring their secrets. Rebuild search and verify equivalent relevant results. Test failed restore and rollback.

**G2 — Portable-package reimport:** import the declared canonical subset into a clean workspace and compare included IDs/relationships/revisions, sources/attribution and selected history/preferences. Explicitly report remapping, unsupported types and omitted sections. Drafts/resume state are not promised unless included by that package's schema/manifest. Content-only export omits personal history and is never described as full recovery. Corrupt/unsupported packages must not modify the live workspace. See the [inclusion matrix](../architecture/data-model.md#portable-packages).

Human-readable Markdown/JSON/CSV and original files permit recovery outside Memoquill even if a future application cannot run its native database. Source version metadata and a documented manifest prevent silent format guessing.
