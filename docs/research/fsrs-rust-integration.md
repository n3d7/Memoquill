# Rust FSRS integration evidence

Update 2026-11-14: the implementation qualification below supersedes the earlier
not-installed status. Historical retrieval limits remain explicit.

Verified on 2026-11-13 using Firecrawl Cloud search/scrape against official, version-pinned docs.rs pages. Scope: an offline recall scheduler, without parameter optimization. No dependency installation, compilation, tests, or project changes were performed during this investigation.

## Dependency choice and cost

The current `latest` documentation identifies **`fsrs 7.0.0`**, published **10 November 2026**, with **BSD-3-Clause** licensing and the official repository `open-spaced-repetition/fsrs-rs`. Pin `fsrs = "=7.0.0"` and commit the resolved lockfile; do not use an unbounded `latest` API assumption. [Crate documentation](https://docs.rs/fsrs/latest/fsrs/), [published manifest](https://docs.rs/crate/fsrs/7.0.0/source/Cargo.toml.orig).

The published manifest uses Rust **edition 2024**, declares **no `rust-version`**, and defines **no feature flags**. Its toolchain file selects `stable`, with a comment that older compilers may fail. Consequently, an exact supported MSRV is **not established by these sources**: validate the project's pinned compiler rather than claiming an upstream MSRV. [Manifest](https://docs.rs/crate/fsrs/7.0.0/source/Cargo.toml.orig), [toolchain](https://docs.rs/crate/fsrs/7.0.0/source/rust-toolchain.toml).

All nine normal dependencies are mandatory: `bytemuck 1`, `itertools 0.15.0`, `ndarray 0.17.2`, `rand 0.10.2`, `rand_distr 0.6.0`, `rayon 1.12.0`, `serde 1.0.228` with derive, `snafu 0.9.1`, and `wide 1.7`. `chrono`, `csv`, `criterion`, and `rusqlite` are dev dependencies. There is no direct Burn, Torch, external model runtime, or network-client dependency in this manifest. This is **not** a transitive dependency audit or measured binary/compile-size result. `default-features = false` cannot remove the optimizer/simulator modules or these mandatory dependencies because there is no scheduler-only feature gate. Scheduling calls can simply avoid optimizer/simulator APIs. [Manifest](https://docs.rs/crate/fsrs/7.0.0/source/Cargo.toml.orig).

## Exact scheduling API

The published source exposes the following signatures and fields (the formatting below is condensed; this example has not been compiled):

```rust
// Parameters is a type alias for [f32].
FSRS::new(parameters: &Parameters) -> fsrs::Result<FSRS>

FSRS::next_states(
    &self,
    current_memory_state: Option<MemoryState>,
    desired_retention: f32,
    days_elapsed: u32,
) -> fsrs::Result<NextStates>

FSRS::next_states_with_elapsed_days(
    &self,
    current_memory_state: Option<MemoryState>,
    desired_retention: f32,
    days_elapsed: f32,
) -> fsrs::Result<NextStates>

MemoryState { stability: f32, difficulty: f32, stability_fast: f32 }
NextStates { again: ItemState, hard: ItemState, good: ItemState, easy: ItemState }
ItemState { memory: MemoryState, interval: f32 }
```

Use `None` for an item with no previous memory state and `Some(previous_state)` subsequently. `next_states_inner` maps ratings **1/2/3/4** to **Again/Hard/Good/Easy**, respectively. It computes four candidate states and intervals; the application selects and persists only the learner's chosen rating. `stability_fast` is the FSRS-7 fast trace and equals `stability` for FSRS-6. Preserve all three values for FSRS-7. [Inference source](https://docs.rs/crate/fsrs/7.0.0/source/src/inference.rs), [MemoryState](https://docs.rs/fsrs/7.0.0/fsrs/struct.MemoryState.html).

The official example calls `FSRS::default().next_states(None, 0.9, 0)`, selects `.good`, then rounds the returned interval to days with a minimum of one before constructing a due timestamp. That rounding is an **application example**, not a requirement to discard fractional intervals; the fractional elapsed-day API is available. [Crate example](https://docs.rs/fsrs/latest/fsrs/), [inference source](https://docs.rs/crate/fsrs/7.0.0/source/src/inference.rs).

## Algorithm and defaults

In version 7.0.0, `FSRS::default()` delegates to `FSRS::new(&[])`. Empty parameters and **34 parameters select FSRS-7**; legacy **17, 19, or 21 parameters select FSRS-6**. `DEFAULT_PARAMETERS` denotes FSRS-7 defaults; `FSRS6_DEFAULT_PARAMETERS` explicitly selects FSRS-6. The 34-value default vector was **not transcribed or independently reviewed** in this bounded investigation. Pinning the crate and recording the actual parameter vector/hash prevents silent default changes during future upgrades. Persist an explicit algorithm version and parameter identity alongside memory state. [Model implementation and version-selection examples](https://docs.rs/crate/fsrs/7.0.0/source/src/model.rs).

For recall-only offline use, prefer the default model with an explicitly configured retention target and no training calls. This is an integration recommendation, not evidence that FSRS covers exercise mastery, course completion, or broader competence.

## Correctness and upgrade strategy

Use the version-pinned upstream `next_states` documentation fixture as an independent numerical reference. For `None`, retention `0.9`, elapsed `0`, the source gives these `(stability, difficulty, stability_fast, interval)` values:

| Rating | State and interval |
| --- | --- |
| Again | `(0.1104, 6.1686, 0.08832, 3.8275626e-5)` |
| Hard | `(2.2395, 5.261278, 1.7916001, 0.5974598)` |
| Good | `(3.9221, 3.5307243, 3.13768, 4.7777247)` |
| Easy | `(11.7841, 1.0, 9.427279, 53.869392)` |

These numbers were read, **not executed**. Reference their exact source/version and use a documented floating-point tolerance in application tests. Add focused application checks for all rating mappings, `Some(previous_state)`, same-day reviews, invalid/nonfinite inputs, due-date conversion, persistence/restart, and rejected cross-course access. Preserve old scheduling events and parameter identity; upgrades should require explicit numerical-fixture review rather than rewriting history. [Upstream inference fixture and validation tests](https://docs.rs/crate/fsrs/7.0.0/source/src/inference.rs).

## Evidence limits

Seven official pages were scraped: current crate docs; version-pinned original manifest; FSRS and MemoryState rustdoc pages; inference source; model source; toolchain file. The FSRS rustdoc extraction was incomplete for useful signatures, so published source supplied them. The crate's crates.io page, full license text, default-vector definition, transitive dependency tree, release notes, and complete upstream test suite were not read. The repository homepage was found through search, but no separate repository page was scraped; docs.rs links this published crate to that repository. No MSRV, offline build, performance, or application integration gate has passed based on this note alone.

## Implementation qualification (2026-11-14)

The complete published `fsrs-7.0.0.crate` was downloaded from
[the crates.io static archive](https://static.crates.io/crates/fsrs/fsrs-7.0.0.crate).
SHA-256 `5cfbc88edd1cc96cec10317192b57cc5a5c50bd48f3144d3920b9bf5da87fa69`
matches the locked registry checksum. Its full [BSD-3-Clause license](https://docs.rs/crate/fsrs/7.0.0/source/LICENSE)
retains Copyright (c) 2023 Open Spaced Repetition, source/binary notice retention,
non-endorsement and the original disclaimer. The unmodified crate is linked;
its actual text and transitive Linux notices are bundled in the dependency inventory.
No application implementation was copied or optimizer invoked.

`src/inference_v7.rs` supplies all 34 actual defaults. The adapter uses that exact
vector, records every value and a domain-separated SHA-256 over its little-endian
f32 bits, and freezes the algorithm identity `fsrs-7/fsrs-rs=7.0.0/daily-v1` in each
event. The test independently pins the vector and the four upstream initial-state
fixtures (relative/absolute floating-point tolerance). Library and fractional-day
API compiled on the project's pinned Rust 1.98.1; this verifies this compiler,
not an upstream minimum supported version. All three memory traces are persisted.

Linked `jiff =0.2.38` and `jiff-tzdb =0.1.9` load pinned bundled TZif bytes directly,
not a host timezone search. Each event retains its timezone, database identity,
day-start, study day, raw interval, rounded daily interval and due day. The daily
policy rounds to at least one civil day; explicit same-day reviews still use the
fractional elapsed-day API. Tests cover Kyiv's DST gap/fold, rollback rejection,
same-day updates and a thirty-year gap. The schedule is a recall estimate, never
proof of retention, problem solving or practical competence.

Only an explicit FSRS rating for unassisted independent/delayed practice recall
updates the model. Unknown help, hints/reveal, recognition, constructed responses,
procedures and assessments are excluded. Attempt/event/state/cursor/receipt are
one transaction. Shared journal order permits native replay of active policy,
clock/prior chain, budgets and FSRS results. Parameter/default identity is fixed;
retention/timezone/day-start changes version subsequent events, budget changes
retain memory/due state, and original events are never rewritten.

Linux reachable Cargo normal/build/test metadata, exact source notices and dependency
tree are retained under `.qualification/m4-fsrs-dependency-evidence/`; the inventory
is bound to the final lockfiles. Twenty-five newly inventoried component/version
pairs use permissive terms (BSD-3-Clause, MIT/Apache, Zlib or Unlicense choices).
No license exception/project relicensing was added. Initial accidentally unfiltered
platform metadata was rejected for missing notices; the corrected Linux closure
contains 333 packages and direct license-evidence verification passed.

Focused algorithm/storage/clock/replay and GUI-free IPC evidence is linked in
[the active M4 receipt](../task-reports/2026-11-14-m4-review-and-assessments.md).
Final full/security/license and native/human qualification remain distinct gates.
