# Memoquill

Work in progress

Memoquill is a planned open-source, subject-agnostic learning application for organizing sources, studying, practicing, and retaining knowledge. Foundation V1 targets offline Linux desktop use, especially Fedora; AI is optional.

Foundation implementation includes the Fedora desktop qualification shell and the M1.2 durable workspace. It demonstrates offline reading/editing, mathematical rendering, passive code, typed Rust calls, native accessibility integration and explicitly saved local reading preferences. Learning content and progress are not yet persistent; the complete learning application remains planned. See the [workspace runtime](docs/architecture/workspace-runtime.md) and [current execution evidence](docs/planning/m1-2-execution.md).

- [Documentation index](docs/README.md)
- [Foundation V1 scope](docs/product/foundation-v1.md)
- [Architecture](docs/architecture/technical-design.md)
- [Capable external-agent/MCP contracts](docs/architecture/mcp-contracts.md)
- [Implementation roadmap and Linear mapping](docs/planning/linear-backlog.md)

For development on Fedora, use the pinned Rust toolchain and Node/npm versions from `rust-toolchain.toml` and `package.json`. Install the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/), run `npm ci`, then `npm run tauri -- dev`. `npm run check` runs the UI/type/build checks, Rust formatting/lints/tests and the optimized dependency regression. `CARGO_BUILD_JOBS=4 npm run tauri -- build --bundles rpm` creates the local RPM; it does not install or publish it. The fixture explicitly warns that edits disappear on window close.

The [M1.1 baseline](docs/quality/fedora-desktop-baseline.md) records passing automated X11/Wayland/IME/Orca checks and remaining manual acceptance. See the [review report](docs/task-reports/2026-10-28-103031Z-m1-desktop-qualification.md) and [native reproduction](tests/qualification/README.md). No public release is implied by this local implementation.

The project license is [MIT](LICENSE). Adapted operational guidance and future dependencies retain their applicable notices; see [third-party reuse](docs/architecture/third-party-reuse.md).
