# M2 authoring: bounded technical evidence

Checked 2026-10-28. Preparation for [KER-99](https://linear.app/kerigan/issue/KER-99/m2-deliver-custom-subjects-material-authoring-and-navigation), not implementation or an accessibility qualification. This adds specific technical details to [technical options](technical-options.md) and [educational UX](educational-ux.md); it does not reopen the chosen stack or claim a learning benefit.

## Retrieval and applicability

One public search and three page retrievals through self-hosted Firecrawl: two useful official pages and one GNOME 404. The local response's `creditsUsed` is accounting metadata, not evidence of hosted billing. No paid research, model job, dependency installation, or application test was run.

Current `package.json` pins ProseMirror model 1.25.12/view 1.42.8/state 1.4.4 and KaTeX 0.19.1. The official pages below are moving documentation, not documentation pinned to those releases. Verify consequential behavior against the installed source and focused tests during authorized execution; these retrievals do not qualify the existing editor fixture or a future M2 adapter.

## New evidence and proposed checks

| Primary source | Verified detail | Engineering implication and limit |
| --- | --- | --- |
| [ProseMirror reference: paste hooks](https://prosemirror.net/docs/ref/#view.EditorProps.transformPastedHTML) | `transformPastedHTML` runs before HTML parsing; `clipboardParser` chooses the clipboard parser, otherwise using `domParser`. | Put an explicit paste policy at this seam and test HTML, plain text and internal editor fragments. A hook's existence does not establish sanitization; DOM parsing/schema acceptance is not a complete URL, attachment or rendering policy. |
| [ProseMirror reference: validation](https://prosemirror.net/docs/ref/#model.AttributeSpec.validate) and [Node.check](https://prosemirror.net/docs/ref/#model.Node.check) | Attribute validators run for JSON deserialization and node checking; `Node.check` checks the node and descendants against the editor schema. | Validate attribute shape and call the appropriate complete document check. Separately enforce Memoquill version, block identity, resource ownership, lengths/depth and link rules in the authoritative application service. Editor conformance cannot establish those domain invariants. |
| [KaTeX options](https://katex.org/docs/options) | `trust` defaults false; `htmlAndMathml` includes visual HTML and accessibility MathML. `maxSize` defaults unbounded; `maxExpand` limits expansion. The options documentation also describes mutable macro state. | Set security/resource options explicitly, bound input length and layout size, and avoid shared mutable macro state across untrusted documents. Preserve source and an understandable error path. MathML output alone is not proof that Orca reads, navigates or edits the result in packaged WebKitGTK. |

Proposed adversarial fixtures: event handlers and active elements in pasted HTML; encoded/disallowed URL schemes; remote image requests; malformed or deeply nested editor JSON; duplicate block IDs; unsupported nodes retained with a warning; oversized or recursively expanded math; and macro state crossing document boundaries. These are future checks, not confirmed vulnerabilities.

## Remaining accessibility evidence

The attempted GNOME page `https://gnome.pages.gitlab.gnome.org/orca/commands_reading.html` returned an actual 404 page (metadata status 404), so it supplies no evidence. The three-page retrieval budget was retained rather than retrying broadly. Reuse the existing educational UX evidence and keep the physical GNOME/human Orca gate open. During implementation, test authoring, selection, caret movement, table navigation, math source/rendered alternatives, paste, undo/redo, save/conflict announcements, IME and focus return in the actual packaged runtime. These are product-specific verification proposals, not outcomes demonstrated by the retrieved documentation.
