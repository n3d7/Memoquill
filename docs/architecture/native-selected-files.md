<!-- SPDX-License-Identifier: Apache-2.0; Copyright 2026 n3d7. -->
# Native-selected package and original files

The main window uses course-bound application ports. A course UUID is resolved to
its service before any native picker awaits; the active catalogue selection is
never consulted afterward. A restored course may retain the same workspace UUID
as its source, so workspace UUID alone is not authority. No command accepts paths,
filesystem handles, SQL, principals or imported execution instructions.

`save_original(courseId, sourceId)` returns `true` after publication or `null` on
cancellation. The serialized service reads the immutable source, enforces 64 MiB,
verifies the managed asset and hashes the complete captured bytes again. The native
adapter captures those bytes and a publication UUID before its save picker opens.
The suggested filename strips path/control characters and is capped at 128 Unicode
scalars. PDF, EPUB, DOCX and HTML can be saved without extraction or viewer support.
Saved bytes are never automatically opened, executed or fetched.

`package_export(courseId, {kind, includeOriginals})` returns `{kind, manifest}` or
`null`. Kind is `nativeFull` (G1) or `portableContent` (G2). Capture completes before
the save picker opens. Originals are opt-in for G2; G1 records missing originals
and completeness. Both preserve the backend's declared content-version history.
G2 omits private learning/recovery data and never claims full recovery.

`package_inspect(courseId)` returns `{reviewId, kind, manifest}` or `null`. Native
selection opens a regular, bounded, non-symlink source and invokes only the bounded
G1/G2 backend readers. Both validate before any course admission. A review is an
opaque in-process capability bound to its originating course; there are at most
eight outstanding reviews. It retains immutable validated staging, not the selected
path. Original package modification after review cannot change the admission.

`package_confirm(courseId, {reviewId, operationId, title, allowIncomplete})` returns
`{kind, report}`. Incomplete G1 needs explicit confirmation. Both backends admit a
NEW course only and bind exact retry operation/title/package identity durably.
Successful admission selects the new course. The renderer refreshes the catalogue
and retains a visible report containing new course ID, source remap and omissions.
Uncertain failures retain the exact request for `Retry same package import`.
`package_discard(courseId, reviewId)` releases staging; foreign-course tokens fail.
Review tokens do not survive application restart: inspect again after restart.

The Linux native adapter opens every selected directory component using `openat2`
with `NO_SYMLINKS|NO_MAGICLINKS`, retains its directory descriptor, refuses resolved
paths inside the complete application data root (catalogue, courses, legacy DB,
originals, journals and recovery roots), then creates a private exclusive staging
file relative to that descriptor. The writer has no destination authority beyond
that selected directory. It syncs staged bytes, enforces the 512 MiB package cap,
then atomically renames within that directory. Failure before rename removes
staging and preserves the previous destination; cancellation creates nothing.
Renaming a selected external hard link replaces only that link. After successful
rename, the final directory sync is best effort: a later sync failure cannot
truthfully be reported as a non-mutating failure. No crash-durability guarantee is
made for unsupported external filesystems.

All five commands are registered in the invoke handler, `build.rs` AppManifest and
`capabilities/main.json`. Real Tauri IPC tests exercise main-window admission,
foreign-window/remote-origin rejection and course-bound review tokens. Unit tests
exercise atomic failure/cancellation, managed aliases, byte-exact original formats,
corrupt/missing originals, both package readers and selection changes. Native
picker interaction and packaged GUI journeys remain separate qualification gates.
