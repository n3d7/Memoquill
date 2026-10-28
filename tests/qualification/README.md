# M1.1 native qualification

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
