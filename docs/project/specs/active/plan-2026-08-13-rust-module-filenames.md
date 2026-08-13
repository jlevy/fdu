---
title: Clear Rust Module Filenames
description: Plan a safe structural refactor and an automated guardrail for self-describing Rust source filenames
author: Joshua Levy (github.com/jlevy) with LLM assistance
---
# Clear Rust Module Filenames

**Date:** 2026-08-13

**Author:** Joshua Levy (github.com/jlevy) with LLM assistance

**Status:** In Review

## Overview

Rename ambiguous Rust source files so a filename remains useful when it appears alone in
an editor tab, search result, stack trace, or code-review attachment.
Preserve every CLI, Python, serialization, and computation behavior.
Add a narrow repository check for the objective parts of the convention after the
behavior-preserving rename is green.

This work lands as a stacked pull request on the file-content-metrics branch.
It does not share a checkout with agents working in the primary repository worktree.

## Goals

- Give each non-root production Rust file a concise, self-describing basename that
  balances subsystem, responsibility, and repository-wide uniqueness.
- Remove production `mod.rs` files in favor of the modern `foo.rs` plus `foo/` layout.
- Preserve existing public item paths and retain `fdu::session` as a compatibility alias
  while introducing the clearer `fdu::watch_session` module.
- Make the refactor mechanical and reviewable through one explicit rename map and exact
  module-path substitutions.
- Preserve byte-for-byte CLI golden output and pass the complete Rust and Python handoff
  gates.
- Prevent the objective failure modes from recurring without trying to automate
  subjective naming judgment.

## Non-Goals

- Split large modules, move functions between ownership boundaries, or redesign APIs.
- Change file-content metrics, query behavior, performance, cache formats, snapshot
  formats, CLI output, Python behavior, or supported Python versions.
- Rename Cargo-defined crate roots such as `lib.rs`, `main.rs`, and `build.rs`.
- Rename fixture source files whose purpose is to model ordinary external repository
  layouts.
- Require every top-level module to repeat the crate name.

## Background

The current production tree has repeated basenames (`cache.rs`, `index.rs`, `types.rs`,
and `mod.rs`) and terse child names such as `code.rs`, `basic.rs`, `parse.rs`, and
`detect.rs`. The directory path disambiguates them to the compiler, but tools often show
only the basename. As the file-content subsystem grows, this makes open tabs and search
results increasingly hard to distinguish.

Rust requires snake_case module names and normally derives module paths from the source
layout. Modern Rust supports a parent in `foo.rs` with children in `foo/`; the Rust
Reference and Rust Book recommend that layout over repeated `mod.rs` files because the
latter are easy to confuse when several are open.
Rust does not require globally unique basenames, so repository-wide recognizability is
an fdu convention layered on top of the language convention.

## Design

### Naming rule

Apply these rules to human-authored Rust files under `crates/*/src`, `crates/*/tests`,
and `crates/*/examples`:

1. Use snake_case and let the physical path mirror the logical module path.
   Do not add a `#[path]` override solely to make a filename look different.
2. Use the modern parent layout: `foo.rs` owns children in `foo/`. Do not add production
   `mod.rs` files.
3. A non-root basename should identify its subsystem and responsibility without
   requiring the directory path.
   Prefer `<domain>_<responsibility>.rs` when a short word would be generic or collide
   elsewhere.
4. Name a module after the concept it owns, not the mechanism currently used to
   implement it. Prefer noun phrases such as `content_cache` and `query_selection`.
5. Avoid catch-all or context-dependent basenames such as `types`, `common`, `utils`,
   `helpers`, `basic`, `code`, `parse`, and `detect`.
6. A concise top-level domain noun such as `cache`, `index`, `scan`, `snapshot`, or
   `watch` is acceptable when it is the unique primary abstraction for that crate.
7. Cargo roots (`lib.rs`, `main.rs`, and `build.rs`) are standard exceptions.
   Golden fixtures are also exempt because their filenames intentionally represent other
   projects. Test and example files should otherwise remain self-describing.

The automated check enforces only rules with an objective signal: no production
`mod.rs`, no forbidden catch-all basename, and no duplicate non-root basename in the
checked Rust tree. Code review remains responsible for whether a new name accurately
describes ownership.

### Rename map

| Current file | New file | Owned responsibility |
| --- | --- | --- |
| `crates/fdu/src/content/mod.rs` | `crates/fdu/src/content.rs` | Public content subsystem root |
| `crates/fdu/src/content/analyze.rs` | `crates/fdu/src/content/content_analysis.rs` | Bounded analysis orchestration |
| `crates/fdu/src/content/basic.rs` | `crates/fdu/src/content/content_basic_metrics.rs` | One-pass physical line and raw-word metrics |
| `crates/fdu/src/content/cache.rs` | `crates/fdu/src/content/content_cache.rs` | Versioned content sidecar persistence |
| `crates/fdu/src/content/code.rs` | `crates/fdu/src/content/content_code_metrics.rs` | Code, comment, and blank-line metrics |
| `crates/fdu/src/content/index.rs` | `crates/fdu/src/content/content_index.rs` | Sparse per-file metrics and directory roll-ups |
| `crates/fdu/src/content/markdown.rs` | `crates/fdu/src/content/content_markdown_metrics.rs` | Reader-visible Markdown metrics |
| `crates/fdu/src/content/types.rs` | `crates/fdu/src/content/content_model.rs` | Content-analysis contracts and metric slots |
| `crates/fdu/src/query/mod.rs` | `crates/fdu/src/query.rs` | Public query subsystem root |
| `crates/fdu/src/query/glob.rs` | `crates/fdu/src/query/query_glob.rs` | Glob grammar and matching |
| `crates/fdu/src/query/parse.rs` | `crates/fdu/src/query/query_values.rs` | Size and time value grammars |
| `crates/fdu/src/query/report.rs` | `crates/fdu/src/query/query_report.rs` | Report model and construction |
| `crates/fdu/src/query/selection.rs` | `crates/fdu/src/query/query_selection.rs` | Selection contracts and evaluation |
| `crates/fdu/src/classify/detect.rs` | `crates/fdu/src/classify/file_type_detection.rs` | Bounded content-dependent classification |
| `crates/fdu/src/types.rs` | `crates/fdu/src/engine_contract.rs` | Shared observation, commit, and provenance contract |
| `crates/fdu/src/session.rs` | `crates/fdu/src/watch_session.rs` | Live watch/query session composition |
| `crates/fdu/tests/watch_session.rs` | `crates/fdu/tests/watch_session_integration.rs` | Watch-session integration behavior |
| `crates/fdu/tests/watermark.rs` | `crates/fdu/tests/scan_watermark_integration.rs` | Scan-watermark integration behavior |

`cache.rs`, `index.rs`, `scan.rs`, `snapshot.rs`, and `watch.rs` remain concise because
each is the unique crate-level owner of that concept.
`lib.rs`, `main.rs`, `build.rs`, and fixture basenames remain unchanged under the
explicit exceptions above.

### Structural substitutions

The rename commit changes declarations, imports, and re-exports only:

- `crates/fdu/src/content.rs` declares `content_analysis`, `content_basic_metrics`,
  `content_cache`, `content_code_metrics`, `content_index`, `content_markdown_metrics`,
  and `content_model`, then re-exports exactly the same public items as before.
- `crates/fdu/src/query.rs` declares `query_glob`, `query_values`, `query_report`, and
  `query_selection`, then re-exports exactly the same public items as before.
- Private `super::types`, `crate::types`, `crate::query::glob`, and
  `crate::query::selection` paths become their precise new internal module paths.
- `crates/fdu/src/lib.rs` declares `engine_contract` and `watch_session`; it keeps every
  existing crate-root item re-export and adds `pub use crate::watch_session as session`
  so existing `fdu::session::*` consumers continue to compile.
- Internal implementation and Python-binding call sites move to `watch_session`, proving
  the new canonical path.
  The watch-session integration target deliberately imports `fdu::session` so the
  compatibility alias remains compile-tested.
- `crates/fdu/Cargo.toml` points the two explicit integration-test targets at their new
  filenames, and architecture/spec references use the new paths.

`repren` may perform these substitutions, but only with an exact, reviewed mapping.
No symbol names, function bodies, tests, or generated expectations change in this phase.

### Policy check

Add `scripts/check-rust-module-names.mjs`, using only Node built-ins, with these small
units:

- `collectRustFiles(root)` walks the checked crate source, test, and example trees while
  excluding fixture data.
- `auditRustModuleNames(paths)` reports forbidden basenames, production `mod.rs`, and
  duplicate non-root basenames; diagnostics list every conflicting path.
- `formatViolations(violations)` provides deterministic, actionable command output.
- `main()` resolves the repository root, runs the audit, and returns nonzero on any
  violation.

Add `scripts/check-rust-module-names.test.mjs` with temporary synthetic trees that prove
clear names pass, standard crate roots may repeat, generic names fail, duplicate names
fail, and fixtures are excluded.
Wire the checker and its test into a dedicated Make target included by `make check`.

### API and compatibility

There are no CLI, Python, serialized-data, metric, or crate-root item changes.
The public `content` and `query` module paths do not change.
`fdu::watch_session` becomes the canonical module, while `fdu::session` remains a public
compatibility alias.
All other renamed logical child modules were private.

The branch remains on Python 3.12-compatible Rust bindings and keeps the existing
`abi3-py312` configuration.

## Implementation Plan

### Phase 1: Baseline and structural rename

- [x] Record the current branch SHA and a green `make test-golden` baseline; the already
  green stacked-base CI is supporting evidence, not a substitute for local validation.
- [x] Apply the rename map with Git-aware moves so history remains reviewable.
- [x] Apply exact module declaration/import substitutions with `repren`, inspect every
  changed hunk, and confirm no function body or expected output changed.
- [x] Run `cargo fmt --check`, Rust unit/integration tests, Python binding tests, and
  `make test-golden`; require an empty golden diff.
- [x] Commit the pure structural refactor separately from policy enforcement.

### Phase 2: Naming guardrail

- [x] Add the dependency-free policy checker and deterministic diagnostics.
- [x] Add focused Node tests for allowed roots, forbidden names, duplicate names, and
  fixture exclusion.
- [x] Wire the check into `make check` without changing unrelated targets.
- [x] Run the focused test, the policy check against the repository, and `make check`.
- [x] Commit the guardrail separately from the rename.

### Phase 3: End-to-end handoff

- [x] Update this plan with the completed rename audit and validation evidence.
- [x] Run `make docs-format`, inspect the complete diff, and rerun `make check` as the
  required handoff gate.
- [x] Run CLI golden tests across all scenarios and verify no `.trycmd` expectation
  changed.
- [x] Run the multilingual repository language/document report as an end-to-end smoke
  check and compare it with the stacked-base output shape.
- [ ] Close and sync all linked beads, push the stacked branch, open the pull request
  against `codex/file-content-metrics-plan`, and watch every CI check to completion.

## Testing Strategy

The rename is a structural refactor executed from green.
Existing unit, integration, property, Python, and CLI golden tests are its
characterization suite.
The golden-test invariant is especially strict: all scenarios must pass without
accepting or rewriting expected output.
The new checker receives focused behavioral tests before it is wired into the global
gate.

Required evidence:

- `make test-golden` before and after the structural rename, with no expectation
  changes.
- Rust tests under the repository Make targets, including explicit watch-feature tests.
- Python 3.12 binding tests through the existing uv/maturin workflow.
- Focused policy-checker tests using Node’s built-in test runner.
- `make check` after each green phase and once more at handoff.
- A real `fdu` language/document report over this multilingual repository.
- Green CI on the stacked pull request.

The local baseline at stacked-base commit `fbb36f892b1f31b4c51ade16403b846e22d080c6`
passed all 92 golden scenarios without expectation changes and passed `make check` on
2026-08-13. The initial golden invocation accidentally resolved an older installed
binary because the temporary Cargo target directory was outside the repository; linking
the ignored `target` path to that build directory made Cargo and tryscript use the same
binary, after which the suite passed cleanly.

After the structural rename, all 92 golden scenarios again passed without changing an
expectation. The complete `make check` gate passed with temporary Cargo artifacts on a
RAM volume and test temporary files on an APFS RAM volume because the host data volume
was nearly full. The gate covered all-feature, minimal, watch, and Rust 1.85 builds;
Python concurrency and the Python 3.12 stable-ABI wheel; documentation; supply-chain and
dependency audits; and the multilingual repository self-check.

The focused policy suite passed all six synthetic-tree cases, and the repository audit
accepted all 35 in-scope Rust files.
The complete `make check` gate also passed after the new check was added, including the
unchanged 92-case CLI golden suite and installed-wheel smoke test.

The final repository-archive smoke test preserved the versioned `fdu.report/2` shape and
returned 16 language rows and four document rows.
Its aggregate code partition was 33,398 code lines, 3,507 comment lines, and 3,755 blank
code lines; the document roll-up was 138,770 words at 250 words per page.
The report included Rust, Python, JavaScript, Shell, and the repository’s small
cross-language fixtures.
Two Makefiles were reported as explicitly unsupported by the code parser, so
`--allow-partial` produced the intended successful partial report rather than silently
claiming complete coverage.

## Rollout Plan

Deliver this as one reviewable stacked pull request on
`codex/file-content-metrics-plan`. Keep the structural rename and guardrail in separate
commits. The compatibility alias makes the only newly clarified public module path
additive; consumers do not need a coordinated migration.

## Open Questions

None. The rename map, compatibility rule, validation gate, and PR stacking boundary are
fixed for this implementation.

## References

- [Rust API Guidelines: naming](https://rust-lang.github.io/api-guidelines/naming.html)
- [Rust Reference: modules and source filenames](https://doc.rust-lang.org/stable/reference/items/modules.html)
- [The Rust Programming Language: separating modules into files](https://doc.rust-lang.org/stable/book/ch07-05-separating-modules-into-different-files.html)
- [fdu Rust engineering quality plan](plan-2026-08-09-fdu-rust-engineering-quality.md)
- [fdu file-content metrics plan](../done/plan-2026-08-12-fdu-file-content-metrics.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
