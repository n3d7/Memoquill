# Bounded source extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Show honest passive extraction of saved TXT/Markdown/CSV/JSON originals.
**Architecture:** Extend the existing course-bound source preview with a pure,
bounded native extraction module. UI renders records/locators as text.
**Tech Stack:** Rust, serde_json, csv 1.4.0, existing Tauri service, React/Vitest.
**Spec:** ../specs/2026-11-09-source-extraction-design.md

## Global Constraints

Linux/X11 only; no GUI/screenshots, external uploads, runtime AI or executable input.
65,536 input bytes; 128 records; 64 fields; no raw filesystem endpoint.
Preserve originals/history/unrelated work; no commits/push/merge authorized.

## Review Focus

- Truncated structured inputs must never report complete extraction (Task 1).
- Duplicate JSON keys must remain in raw records (Task 1).
- Headerless CSV must retain its first record and multiline locators (Task 1).
- Preview retries/restart must not change source receipts/revision/bytes (Task 2).
- Hostile markup and unknown extraction versions must render inertly (Task 3).

### Task 1: Bounded native handlers

**Files:** Create sources/extraction.rs and sources/extraction/tests.rs under
src-tauri/src/workspace; modify sources.rs/types.rs and Cargo.toml/lock.
**Interfaces:** `extract(filename: &str, bytes: &[u8], truncated: bool) -> SourceExtraction`;
types SourceExtraction, ExtractionRecord, ExtractionLocator and enums in types.rs.

- [x] Write failing behavior tests with literal UTF-8/CSV/JSON/hostile fixtures,
  malformed/deep/truncated input and exact record/field boundaries.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml --lib sources::extraction`;
  expect missing behavior before implementation.
- [x] Implement versioned passive extraction and pinned csv dependency; preserve
  raw JSON strings and enforce bounds before processing structured input.
- [x] Run focused tests; expect all pass. Record evidence in the ledger.

### Task 2: Verified source-preview integration

**Files:** sources/preview.rs, sources/tests.rs.
**Interfaces:** SourcePreview gains optional extraction; existing source/text/hex/
truncated retain their meaning. Parser receives only the verified bounded bytes.

- [x] Write integration test: saved .csv returns fields/locator; repeat/restart
  returns equal extraction with unchanged source list, receipt and asset bytes.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml --lib sources::tests`;
  expect missing extraction behavior.
- [x] Compute extraction only after verified asset read; populate both raw and
  extracted preview responses without new commands or mutations.
- [x] Run source tests; expect all pass.

### Task 3: Accessible passive preview display and final gates

**Files:** src/application/sourceTypes.ts, src/SourcesPanel.tsx and tests;
create src/sources/ExtractionPreview.tsx as the focused view.
**Interfaces:** optional SourcePreview.extraction mirrors native v1; unsupported
future contract versions display a fallback rather than interpreting records.

- [x] Add failing UI test for visible partial/failed/attachment status, locators,
  CSV fields, hostile markup and unsupported contract version.
- [x] Run `npm test -- src/SourcesPanel.test.tsx`; expect missing UI behavior.
- [x] Add typed view with status, diagnostics and bounded plain-text records.
- [x] Run focused test and normal build/coverage/fmt/Clippy/native/startup gates;
  expect pass, preserving existing ignored native tests and bundle warning.
- [x] Obtain independent whole-change review; fix material findings with RED/GREEN.
- [x] Update report/HANDOFF with completed slice and remaining KER-100 criteria.

No commit steps: current user authorization covers local implementation only.
