# Local course reads with MCP

Memoquill's Linux/X11 desktop is the only owner of the course services and
storage. The installed `memoquill mcp` command is a stdio bridge using the
official Rust SDK `rmcp` pinned to 3.11.0. The bridge communicates with the
desktop over a private Unix-domain socket. It requires no provider account,
external credential, background daemon, or listening network port.

## Pairing and permissions

1. Open Memoquill and select the course to share. Open **Connections** and
   choose **Enable local connections**.
2. Start the MCP host with command `memoquill` and argument `mcp`. Run
   `memoquill_connection` in the host and copy its pairing challenge.
3. Refresh Connections and enter the challenge. Select the data to disclose
   and approve the request for the displayed course. A pending challenge
   expires after five minutes.

The UI displays the local bridge process ID. A challenge identifies the
pending socket and cannot transfer the grant to another connection. Grants
last up to one hour and expire on disconnection or desktop shutdown. A course
change leaves an existing grant tied to the course originally agreed. Use
**Revoke** or disable Connections to end access. Closing the desktop requires
a fresh pairing after restart. Recent connection events are kept locally in
memory for the desktop session. They contain event types, times and course IDs.

An approved host can copy or transmit disclosed material. Share only material
appropriate for the host and its destination. UNIX ownership and peer-UID checks
constrain the desktop's broker. The operating-system account itself is the local
trust boundary.

| Permission | Read scope |
| --- | --- |
| Course content | Active course structure, component schemas, published lessons and paged normalized document text. |
| Extracted source records | Active source revisions bound to the published lesson, including locators and available coverage. Requires course content. |
| Original source bytes | Hex-encoded 16 KiB chunks verified against the source digest. Requires source records. |
| Aggregate lesson progress | Coverage, counts and aggregate judgment categories. |
| Raw learner responses | Response values and published revision IDs by page. Requires progress. |

## Contracts

Application contract version 1 is independent of MCP protocol revision.
The adapter accepts MCP `2026-07-28` per-request metadata and the
`2025-11-25` and `2025-06-18` legacy handshakes through the SDK. The capability response
identifies the approved course and permissions. Tool schemas are available
through `tools/list`.

- `memoquill_connection` returns approval status or a challenge.
- `memoquill_capabilities` returns the approved course and read permissions.
- `memoquill_structure_get` and `memoquill_content_search` return bounded
  pages. Continue with their opaque `nextCursor`, using the same kind or
  search query. A cursor belongs to its connection and revision. Refresh
  from the first page when the cursor expires or the revision changes.
- `memoquill_registry_list` and `memoquill_lesson_get` expose component and
  action schemas, published node identities and allowed source bindings.
- `memoquill_resource_read` and the `memoquill://resource/{id}` template
  open the same authorized normalized text. Continue with its revision and
  `nextOffset`; offsets count UTF-8 bytes at character boundaries.
- `memoquill_source_get` returns a bounded preview of extracted records with
  the lesson binding revision and locators. Extraction and source-use coverage
  describe different facts. `memoquill_source_original` reads original bytes
  with the separate permission.
- `memoquill_progress_get` reports aggregate counts. `includeResponses`
  opts into raw response payloads when the separate permission exists.

Use `memoquill_connection` after a transport failure. A missing desktop or a
disabled broker returns `app_not_running`. A known unauthorized ID and an
unknown ID yield the same denial where applicable. Protocol data stays on
stdout and diagnostics go to stderr. Draft authoring and delegated live changes use separate permissions.

## Lesson drafts

Use `memoquill_lesson_draft` with `unitId`, a private random version-4 UUID
as `ownerKey`, and an `action` object. A new draft requires the exact
`expectedHeadRevision` when based on a saved lesson. Omit it for an empty
lesson. A separate draft grant is required. Reconnecting requires approval
for the course and draft operations again. Use the same owner key to resume.

| Action | Required fields |
| --- | --- |
| `create` | `operationId`, optionally `expectedHeadRevision` |
| `list` | None |
| `read` | `draftId` |
| `edit` | `draftId`, `revision`, `operationId`, `operations` |
| `validate`, `preview` | `draftId`, `revision` |
| `discard` | `draftId`, `revision`, `operationId` |
| `materials` | `offset` |
| `sources` | `resourceId`, `offset`, and the source grant |

Edit actions reuse typed lesson node, activity, interaction, layout and
binding operations. New material pins must target an active resource already
linked to the unit. Source bindings require the source grant, and pin a source
under a linked resource. Draft read and preview recheck the sources
and active resources under the current grant. Original bytes remain governed
by their own grant. Validation reports repairable field issues and cannot
verify factual claims.

The trusted desktop can inspect and reject a draft for the current lesson.
Publication requires approval of the exact composition revision, hash, base
and dependency closure in the existing lesson transaction. It retains earlier
lesson revisions and learning history. External authorship stays visible.

## Course groups and resumable work

Use `memoquill_course_draft` with the same owner key and one typed action.
The `create` action accepts a caller-generated `draftId`, `title` and the
current `baseRevision`. Read the base revision from a first structure page.
An action affects one course and one durable operation group.

- `append` accepts up to 64 operations for the expected draft revision.
  A batch is a sequential checkpoint with a version-4 operation ID.
  Use the same operation ID and identical input after an uncertain response.
  Read or list to resume. A group stores at most 128 checkpoints, 512
  operations and 2 MiB.
- `read` pages through proposed operations. `structure` pages through the
  effective course overlay. `validate` reports the failing operation index,
  if known. Pending content stays unpublished.
- Each organization operation wraps one typed `Command` as
  `{"type":"organization","command":{...}}`. The ordinary course
  organization service validates subject, goal, collection, unit, resource,
  path, placement, tag and prerequisite or related links.
- A document operation uses `type: "document"`, `resourceId`,
  `expectedRevision` and the normalized `document`. Use it for staged
  structured text and exercise material. Existing originals are accessed
  through the lesson source services. The host has no caller-path upload.
- `fork` can replay an authorized prefix of checkpoints on a new draft and
  base revision. `submit` seals the group for review. An unapplied draft or
  submission can be withdrawn with `discard`.

Course proposals apply an entire reviewed group, including its document
revisions, in one workspace transaction. Divide large proposals into
independently reviewable groups. An approved group fails atomically when
its base changes, a pin is stale, or validation rejects one operation. A
lost acknowledgement can be recovered with the same operation ID.

The separate **Delegate safe course additions** grant permits a submitted
group to call `apply_delegated` with its exact revision and hash. It admits
only safe additions and revisions or organization of items created within
that group. Changes to existing items require desktop review. It provides
32 operations per connection grant, expires with the connection, and shares
a transaction boundary with the writer for budget and revocation checks.
Lifecycle events are in the local Connections panel and receipts are durable.
Undo is explicit desktop compensation for unmodified structural additions.
It archives or unlinks those additions and declines if later changes made
the group unsafe to reverse.

Schema 11 carries local drafts, receipts and external document origin fields
in native full backups. Content-only portable exports exclude the authoring
work history. Use a verified schema 10 native backup to return to a schema 10
build after migrating.

[Roadmap](../planning/roadmap.md) · [Agent contracts](mcp-contracts.md)
