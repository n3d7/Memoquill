# M1 native qualification

## M3 isolated X11 qualification

`m3-native-probe.sh` runs the M3-specific journey in the existing private desktop
image, under `dbus-run-session`, with a fresh task-only XDG profile and a bounded
X11 run. It refuses other targets and missing `m3-journey.py`; do not count the
runner alone as a passed import/search/recovery journey. Build/install the final
M3 candidate in a fresh task image and keep `--network=none`, resource limits and
the reviewed private desktop/SELinux boundaries described below.

Opening the app requires current user approval. Screenshots are off by default;
use `MEMOQUILL_M3_SCREENSHOTS=1` only with explicit permission for that run, and
capture only synthetic task-profile Memoquill content. The user approved GUI and
temporarily screenshots for the November 9 M3 task; this is not global authority
for future runs. Human audible speech/reflow remains separate M7 acceptance.
Record final source/RPM/image digests and actual results in `docs/quality/m3-qualification.md`.

## M2 X11 functional exit

For the M2-only functional pass, set `MEMOQUILL_M2_AUDIT=1`,
`MEMOQUILL_STORAGE_AUDIT=1`, `MEMOQUILL_M2_UI_ONLY=1` and
`MEMOQUILL_CAPTURE_SCREENSHOTS=0`, and pass `x11` to `native-probe.sh`.
UI-only mode rejects other backends and non-M2 runs. It preserves AT-SPI and
real keyboard/IBus interaction but does not start Orca or run speech assertions;
`speech-x11-skip.json` explicitly records `not_run`. Default speech qualification
is unchanged. This pass cannot establish human listening or release AT acceptance.

Use a fresh private profile and the resource/security bounds below. When testing
an extracted candidate instead of the image's installed package, prepend its
`usr/bin` to `PATH` so the journey's restart executes the same candidate. Record
its RPM/source digests. The M2 journey does not call screenshot helpers. It also
checks the empty named text control's accessible attribute run when exposed;
a separate named block control is valid. See the [M2 ledger](../../docs/quality/m2-qualification.md)
for the recorded candidate, passes and explicit release-level omissions.

## Original M1 procedure

This is a private test desktop, not a deployment profile or a test against the workstation session. Fedora 44, rootless Podman, the pinned Node/Rust toolchain and Tauri build prerequisites are required. The recorded run used host UID/GID 1000. Adapt user mapping for another account before running; do not change global SELinux policy or mount host desktop/D-Bus sockets.

From the repository root, build the RPM and prepare task-only inputs:

```sh
npm ci
npm run check
npm run tauri -- build --bundles rpm
mkdir -p .qualification/native-input .qualification/native-results
cp tests/qualification/*.py tests/qualification/native-probe.sh .qualification/native-input/
cp src-tauri/target/release/bundle/rpm/Memoquill-0.1.0-1.x86_64.rpm .qualification/native-input/
podman build -f tests/qualification/Containerfile -t localhost/memoquill-m1-qualification:20261027 .
podman build -f tests/qualification/Desktop.Containerfile -t localhost/memoquill-m1-desktop:20261027 .
podman build --network=none -f tests/qualification/Installed.Containerfile -t localhost/memoquill-m1-installed:20261027 .qualification/native-input
```

The base image digest is fixed; Fedora repository packages can change. Record installed versions/image ID and rerun qualification after rebuilding. These commands do not promise byte-identical images.

Run each backend with a new profile/private bus:

```sh
podman run --rm --timeout=100 --network=none --userns=keep-id \
  --cap-drop=ALL --security-opt=no-new-privileges \
  --security-opt=seccomp=unconfined --security-opt=label=disable \
  --pids-limit=512 --memory=4g --cpus=3 \
  -v "$PWD/.qualification/native-input:/input:ro" \
  -v "$PWD/.qualification/native-results:/evidence" \
  localhost/memoquill-m1-installed:20261027 \
  dbus-run-session bash /input/native-probe.sh x11
```

Replace `x11` with `wayland` for native GDK Wayland under nested Weston. Add `-e MEMOQUILL_NETWORK_TRACE=1` before the image to trace socket creation/outbound calls through app children. Do not add `:Z` while parallel runs share mounts. Outer label/seccomp exceptions are limited to this disposable rootless desktop profile so nested GTK sandboxing can operate; global SELinux remains Enforcing. Under Codex, Podman uses the reviewed host-execution path because the restricted process's SELinux transition failed. See the [baseline and limits](../../docs/quality/fedora-desktop-baseline.md).

The runner exercises actual keyboard/AT-SPI/IBus, then waits for orderly Orca shutdown before checking its buffered speech log. A live-log timeout is not reliable evidence of silence. Speech assertions establish presentation requests, not audio or universal math/language support. `MEMOQUILL_SPEECH_AUDIT=1` optionally records live-region events for diagnostics; those events are not the final speech oracle.

Output is ignored under `.qualification/native-results/`: interaction/speech JSON, screenshots, native protocol/Orca logs and optional network trace. Preserve it until review. The repository documentation snapshot in `docs/quality/m1-desktop-evidence.json` identifies the local candidate without claiming publication.

Unit/static/tool checks and their results are summarized in the baseline. `rust-mutation-test.sh` supplies assets/private crate entry files only to mutator's worker overlay; do not count a build failure as a killed mutation. No source/package installation on the host, human listening, physical GNOME certification, release, commit or tracker closure is implied by these automated runs.

## M1.2 durable-workspace audit

Use the same existing desktop image and a fresh installed image/profile for the
M1.2 RPM; keep the older M1.1 evidence separate. Current reproduced artifact and
results are in [workspace qualification](../../docs/quality/workspace-qualification.md).
Prepare `.qualification/m12/native-input` with the RPM plus this directory's
Python scripts and `native-probe.sh`, then build with `Installed.Containerfile`
and tag `localhost/memoquill-m12-installed:20261028`. Use the same bounded run
above with the M1.2 input/results paths, `--timeout=180` and
`-e MEMOQUILL_STORAGE_AUDIT=1`; run X11 and Wayland separately.

The additional journey explicitly saves Dark/200%, closes the original window,
waits for the retained owner lock to release, restarts, checks saved settings
and the reset temporary material, and requests readable workspace status. X11
uses Alt+F4; nested Weston lacks that close binding, so Wayland activates GTK's
real accessible Close control. A forced signal is not the graceful-exit oracle.
The shell requires the original process's exit code0. The restarted child gets
an explicit backend and Wayland debug logging; its own restart log must contain
Wayland protocol evidence. Screenshot capture restores its temporary X11
environment instead of changing the backend inherited by later children.

The test wrapper defaults to one Rust harness thread because concurrent forked
helpers can briefly retain unrelated flock descriptors until exec/close. Explicit
process contention/crash tests still exercise real concurrency. Mutation workers
materialize private Rust source files and reuse passive assets; source build
failures must not be counted as behavioral kills.
