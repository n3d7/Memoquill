# Course workspace runtime

Post-M2 KER-127/128/129. See the [design and alternatives](../superpowers/specs/2026-11-08-course-workspaces-design.md)
and [extension foundation](course-extensions.md). Main delivery is Linux/X11;
native window/file behavior stays in adapters and other platforms remain deferred.

## Ownership and migration

CourseManager owns a private, exclusively locked version-1 catalogue at
`courses/catalogue.sqlite3` beneath application-local data. It stores only course
metadata, selection and initialization status. A course ID is distinct from its
database-issued workspace ID and the guest actor. Version 1 admits 32 courses.

The old `workspace` root is adopted in place as one course. Existing Subjects
remain nested organization; Units remain lessons. No resource, placement, source,
draft, composition, revision, annotation or history is split, copied or rebound.
This preserves M2 shared relationships without creating mutable cross-course
links. A fresh install starts My course; an existing workspace is Imported
workspace. A later explicit split/copy operation requires its own design.

New roots are generated from catalogue UUIDs at `courses/data/<courseId>`;
callers cannot supply paths. Every root uses the existing workspace schema,
writer queue, original assets, receipt store and recovery guard. Existing SQL
migrations 001–005 are unchanged. Identical object/operation IDs in two databases
do not confer authority or make them aliases.

Catalogue bootstrap alone never opens/upgrades the legacy database. Normal course
service open preserves supported content/identity through the existing migration
pipeline. Unknown/newer/corrupt catalogue data is retained for recovery. A missing
or empty catalogue beside existing managed course roots must fail recoverably,
rather than inventing replacement ownership. Unknown roots are never adopted or
removed automatically. An existing valid catalogue can coexist with retained
abandoned roots; they are not visible course authority.

Initialization is recorded before the first writer starts. Missing storage for a
previously opened/adopted course is a recovery condition, not an empty replacement.
A crash between this record and initial database publication conservatively
requires recovery. This choice favors preservation over automatic reset. There
is no new backup restore/import UI in this step.

Shutdown refuses further service/create/select admission, drains all started
writers and releases catalogue ownership only after successful drain. The stopped
manager cannot acquire new mutation authority. A failed/unconfirmed drain retains
the ownership guard. All services are bounded by the course limit; no eviction
silently drops pending writes and no daemon/scheduler is introduced.

## Application and UI boundary

Every data-bearing native command requires courseId. Native handlers resolve the
matching service before any await or file picker; there is no process-wide
current-course fallback. Existing inner workspace/lesson/reference validation
still runs. Reads, counts, searches, originals, drafts, receipts and preferences
consult one course database only. Native context is constructed for the trusted
main window; unverified external contexts remain denied.

Immutable port factories bind courseId at construction. Unbound compatibility
facades fail closed. The production CourseApp injects all content/source/workspace/
lesson ports through ApplicationProvider, independently per course. Future agent
grants must bind course/workspace/actor/operations and object/range scope through
the same services; no live agent transport or grants are implemented now.

The functional sidebar creates/selects courses and persists the last selection.
Visited course trees stay mounted under stable keys and are hidden/inert when
inactive. Selection, form text, undo, preview responses and exact pending retries
remain in their originating tree. Inactive preferences cannot alter global text
scale; hidden dialogs/file-picker completion cannot steal active-course focus.
Durable lesson drafts use existing persistence; unsaved browser memory retains
the existing close warning and is not claimed crash-durable.

Confirmed catalogue rejections release the pending UI action so the learner can
correct input or navigate. Unconfirmed outcomes retain the exact request for
safe retry; navigation never silently retargets that request to another course.
Full visual design remains deferred. DOM/native IPC tests establish software
behavior, not packaged X11 keyboard/speech or human accessibility acceptance.
