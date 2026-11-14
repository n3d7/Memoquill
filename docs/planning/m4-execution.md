# M4 execution contract

2026-11-13: the user approved the full scope in `m4-readiness.md` and instructed
implementation. Deliver M4.1/KER-102 with KER-131, then M4.2/KER-103. No commit,
push, merge, learner-content modification or GUI launch is implied.

The first storage slice reuses course-scoped `ApplicationService` and its single
writer. A study session pins one committed composition revision and its activity
bindings, never an author draft. Its ordered activities, mode and feedback policy
are immutable. A saved response is a durable draft; hint/reveal transitions are
observed monotonically. External help stays explicitly declared or unknown.
Assessment mode withholds hints/solutions until completion.

Every mutation has a caller-generated operation UUID and expected session
revision. Identical retries return their original receipt; changed payloads using
the same UUID conflict. Submitting inserts an immutable attempt and separate
deterministic/self-report judgments, advances the cursor, resets the draft and
stores the receipt in one SQLite transaction. Only acknowledged commits are
reported successful. Appending a correction retains the original judgment.
No supported delete operation removes study history. Preview never calls study
services. Definitions/relevance are checked against pinned immutable revisions.

Schema migrations add study records without altering existing migration bytes.
Native admission validates the new records semantically; content-only portable
exports continue to omit personal history explicitly. M4 must add opt-in history
export and verify native/full recovery before stage completion.

Next slices expose the same services through bounded native DTOs and the offline
study UI, then add recall-only scheduling and scoped assessment selection. Keep
unknown types inert; test replay/conflict, cross-unit/course scope, restart and
transaction failure before broad qualification. Existing learning evidence is
reused; software checks do not establish educational effectiveness.

## Environment repair completed during startup

Serena had an empty `language_servers` list and a rustup proxy without the
rust-analyzer component. Enabled `rust` and `typescript`, installed the missing
component and Serena-pinned TypeScript dependencies, and used the existing local
dashboard API to activate them in the current MCP session. No GUI or server
restart was needed. Native MCP symbol overviews now succeed for Rust and TS.
Only the language-list key changed; backup/manifest is under
`.qualification/serena-repair-20261113T173004Z/`. Health-check passed; logs:
`/tmp/memoquill-serena-health-check-after.log` and
`.serena/logs/health-checks/health_check_20261113-193052.log`.

## Evidence continuation

New attempts record the first acknowledged draft save and submission as optional
UTC Unix milliseconds; judgments record their observation time. These are wall
clock observations, not active time, latency or proof of delayed retention. Clock
rollback is retained, never silently clamped into a positive duration. Older
records retain unknown times, provenance and instructional context.

A session pins its instructional context. Context labels describe the learner's
chosen check; they do not verify independence, delay or transfer. New judgments
pin composition/activity revision and a canonical definition digest independently
of self-rating, assessor version and uncertainty. Self corrections may explicitly
mark an outcome disputed, append to the latest self-report and preserve all prior
records. The UI exposes that operation with existing uncertain-write recovery.

Model identity/version, prompt/rubric version and reported uncertainty have typed
reserved representations. M4 admits no model judgments or numerical confidence
through native history or mutation APIs; M6 must define scoped authorization and
admission before activation. No provider SDK or automatic competence conversion.
Worked examples/completion still require actual instructional flow, not a label.

## Recovery, export and cloze continuation

M3 schema-7 native packages are validated against their exact trusted schema and
remain immutable during admission. Restored copies use the normal schema-8
migration and backup. This is a qualified 7/8 path, not generic old-schema support.
Portable export adds an explicit, initially unchecked history choice and a required
studyHistory section version 1. Included saved study drafts and earlier receipt
snapshots are private learning data and are named in consent/summary. Default
content-only export omits them. Native reconstruction validates every history row;
future/undeclared sections are rejected and legacy content-only declarations work.

Bounded cloze uses recall activity version 2: one literal {{answer}} token and a
1024-byte answer/response bound, rendered as [blank]. It is not an executable
format or a new activity family. Plain recall version 1 remains unchanged.
Instructional context labels remain distinct from actual phased instruction.
FSRS investigation is in docs/research/fsrs-rust-integration.md; no dependency or
scheduler is activated by that evidence note. See the continuation receipt for
current checks and outstanding acceptance.

## M4.2 implementation decisions (2026-11-14)

M4.1 final quick passed with audit at .qualification/verify/20261114T001015-1ee45633/summary.json; previous frontend/broad Rust and scoped review evidence remain in the instructional receipt. Proceed with M4.2 implementation; milestone/native/human acceptance remains separate.

Pin fsrs =7.0.0 and explicitly persist its 34 FSRS-7 default parameters, their bitwise hash, algorithm/crate version and schedule generation. Link the permissive unmodified crate; do not activate optimization. Use pinned jiff with bundled IANA data and explicit per-course timezone/day-start policy, without host timezone lookup. Due queues use civil study-day ordinals; FSRS elapsed time uses observed UTC milliseconds. Round generated intervals to at least one civil day (daily-review policy), while allowing explicit same-day reviews. Reject backward clocks for scheduled reviews rather than invent positive elapsed time; retain raw wall-clock observations elsewhere.

Only an explicit Again/Hard/Good/Easy rating on an independent, unassisted practice recall (plain or bounded cloze) enrolls/updates scheduling. Generic self-ratings, choice recognition, assisted work, assessments and practical tasks never update FSRS. Attempt/event/state/receipt/cursor commit together. Content revisions get distinct states; old events remain immutable. Retention/timezone/day-start changes increment generation for subsequent events; budget changes preserve memory and due state. Daily new/review limits suppress offered work without deleting overdue history. Practical reminders are a separate explicit manual/fixed-day plan, never an FSRS competence estimate.

Scoped topic/subject assessments freeze the resolved unit/binding/activity/composition revisions and feedback policy at creation. Report coverage by family/outcome/help/unknowns, not a composite mastery score. New records participate in exact-version native admission and opt-in portable history; preserve schema-7/8 read/restore qualification when adding the next migration.

## Integrated M4.2 candidate

Frozen assessments use active lesson placements in the chosen subject or topic
and descendant topics. Unplaced units are outside scope; duplicate placements
are deduplicated. At most 64 units/256 assessable activities are frozen with
composition/activity pins, objective snapshots, assigned session IDs and explicit
omissions for instructional/unsupported/unpublished content. Scoped native Start
resolves pins from that frozen plan, never caller unit/mode overrides. Original
Start cannot bind an assigned assessment session. Feedback is a local presentation
policy (not anti-cheat secrecy for learner-owned source): judgments and aggregate
grading stay withheld until every part completes. Coverage separates recall,
recognition, constructed response and practical self-report, unattempted/unknown
assistance, deterministic checks and latest self-assessments, without a mastery
aggregate. Existing author content and raw histories remain untouched.

Schema9 adds review/assessment data without changing migrations1–8. Native admission
preserves exact schema7/8 verification and migrates only restored copies; schema8
cannot declare schema9-only review/assessment links. Portable history2 carries an
explicit fixed whitelist including review journal and frozen assessments; history1
remains accepted for prior study-only payloads, and content-only exports omit all
personal schedules and assessments. Schema9 needs 132 trusted schema objects;
enumeration remains bounded to160 and exact DDL comparison remains mandatory.

Per-lesson assessment starts also select independent supported pins explicitly and
announce instructional exclusions. This uses the same native selection contract;
requests including an instructional item are still rejected. Revision-conflict
recovery retains local responses separately, reloads authoritative revision/help,
and restores only onto the same pinned activity. Uncertain writes still retry the
original operation UUID before any alternative mutation.
