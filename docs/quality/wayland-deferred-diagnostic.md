# Deferred Wayland text-attribute crash

Diagnosis stopped when the user selected X11-only core delivery on2026-11-06.
This is retained evidence for [KER-124](https://linear.app/kerigan/issue/KER-124/deferred-after-m8-linux-wayland-support-and-qualification),
not a requirement to continue Wayland work during M1–M8. No CSS workaround was
applied and no application source changed in this diagnostic slice.

Candidate: b67f0fd RPM payload92639334369726e9cc9ae9dcdbe3394cd306d040bd5c713e0531ce7e36203ec8.
Existing Fedora44 image1e00a0487ca37eadda1f31371e3c9e620f95e7beaa7cb1583587195d6c483afb:
WebKitGTK2.54.2, GTK3.24.52, glib2.88.4, Orca50.3, strace7.3 with libdw unwinding.
Private rootless containers, network disabled, no host display/bus, at most4GiB/
3CPUs/512PIDs and60–75 seconds; existing documented outer GTK label/seccomp
exceptions only. No host package/config/security change or screenshot capture.

- Memoquill without Orca: the synthetic New subject input retained text, no SIGABRT.
- Same application with Orca: input became unavailable. strace captured self-SIGABRT
  in WebKitWebProcess through WTF::CrashOnOverflow and
  WebCore::AccessibilityObjectAtspi::textAttributesWithUTF8Offset.
- Orca debug identifies a label's embedded empty input, requesting attribute run
  at offset11 (label text New subject followed by an object-replacement character).
- A direct GetAttributeRun(11,true) on that actual application label reproduces
  the same stack without Orca. GDK_BACKEND=wayland and the DMA-BUF override1 were
  recorded. This narrows the trigger to an accessibility query, not GPU warnings.
- Simple GTK/WebKit HTML controls, including the label query and an empty input
  with representative flex styling, did not reproduce the crash. The minimal
  control's Orca target handling was not observed; it is not a passing speech audit.
  Application-independent reproduction and a safe workaround remain unproven.
- One early direct-query attempt stopped on a different label lacking a text
  interface; the corrected selector skips it. A zero runner exit does not override
  inputMatched=false or the recorded abort in the completed application probes.

The related [WebKit PR67179](https://github.com/WebKit/WebKit/pull/67179) describes
out-of-object text-run bounds and a checked-overflow failure, with a fix landed on
its2.52 branch. This is a useful related lead; it does not establish inclusion or
resolution for the installed2.54.2 package. No dependency downgrade/update or
upstream patch was applied. MESA/ZINK warnings also occurred in successful controls.

Synthetic traces, results and experimental helpers are retained ignored under
.qualification/m2-composition/wayland-stack/. Future work should minimize the
actual failing DOM/layout/query and verify an engine fix or bounded accessible
adapter change; do not repeat broad cold-Orca trials or disable accessibility.
