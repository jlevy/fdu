# Feature: Rust Engineering Quality Hardening

**Date:** 2026-08-09

**Author:** fdu project

**Status:** Active — two P0 merge blockers open

## Overview

Apply the useful parts of the Rust Porting Playbook to fdu as an existing, native Rust
codebase. The goal is not to make fdu resemble the playbook mechanically.
It is to find the few remaining contracts whose current shape can hide failure,
constrain future implementation choices, or make a pre-release API harder to support.

This plan comes from a complete review of every document under `guidelines/` in
`jlevy/rust-porting-playbook` at commit
[`d24760a3fbd2951c730a199269aeb082abb46a42`](https://github.com/jlevy/rust-porting-playbook/commit/d24760a3fbd2951c730a199269aeb082abb46a42).
The checkout was acquired through the tbd third-party-repository workflow and treated as
read-only data. Its agent hooks, editor/agent configuration, GitHub workflow, local git
configuration, and instruction files were inspected without being executed.
No forbidden zero-width, soft-hyphen, or bidirectional-control text was found.

The resulting work is additive to the [Phase 1 plan](plan-2026-08-08-fdu-phase-1.md).
It protects that plan’s packed-record, reducer, snapshot, watcher, CLI, and publishing
changes rather than replacing or rescoping them.

## Current Status

The epic is `fdu-dxee`, a child of the Phase 1 epic `fdu-qfz6`. Two independent P0 bugs
are the current PR #1 merge blockers:

- `fdu-ad45`: executable-dependency cool-off, provenance, and CI trust controls;
- `fdu-nlh8`: whole-batch path validation before any index mutation.

Both block the final merge-approval bead `fdu-sn43` in the Phase 1 plan.
The remaining P1 and P2 work protects pre-release refactors and publishing; it does not
outrank those two fixes.

## Goals

- Restore the repository’s stated 14-day executable-dependency cool-off and enforce it
  in the handoff gate
- Make malformed producer input fail before any index mutation instead of looking like
  an unchanged observation
- Keep synchronization primitives and unnecessary allocations out of the supported Rust
  API
- Add refactor-oriented state-machine and persistence tests before the Phase 1 storage
  and reducer rewrites
- Pin the normal Rust toolchain and prove every supported feature and MSRV combination
- Preserve filesystem-native path identity through classification and language bindings
- Make CLI rendering safe for the deepest tree the index can represent
- Finish package, compatibility, and release evidence before the first public release

## Non-Goals

- Treat fdu as a port of a Python or other source implementation; no such parity source
  exists
- Duplicate the completed end-to-end CLI golden-test plan or replace its exact output
  contract with snapshots that normalize more behavior
- Build the Phase 1 fast walker, packed records, reducer registry, block snapshot,
  watcher backends, JSONL mode, or release workflow in this workstream
- Add async, unsafe code, another crate, a general plugin framework, or a trait without
  a demonstrated consumer
- Adopt every suggested tool in the playbook; a tool is added only when its evidence is
  worth its dependency and maintenance cost
- Claim a coverage percentage, performance result, platform guarantee, or semver promise
  that has not been tested

## Background

### Baseline Evidence

The existing engineering floor is already high.
On 2026-08-09, `make check` passed with:

- 125 Rust unit and integration tests across the workspace
- 2 doctests
- 25 built-binary golden scenarios across four stateful CLI sessions
- 95 core-library tests with default features disabled
- Clippy pedantic with warnings denied and unsafe code denied in the core crate
- rustdoc warnings denied, Cargo and npm audits, and an installed abi3 wheel smoke test

The suite has no ignored tests, no sleeps, and no untracked `TODO`, `FIXME`, or `HACK`
markers.
CI already runs the Rust and golden suites on Linux, macOS, and Windows, and the
core library’s no-default-feature path has its own job.
Typed library errors preserve I/O causes, the CLI uses `anyhow` only at its reporting
boundary, snapshots fail closed, and the optional watch layer remains removable.

The review therefore did not recommend a package split, a new error framework, async,
more broad lints, or a rewrite of working test infrastructure.

### Findings That Require Work

| Severity | Evidence | Required correction |
| --- | --- | --- |
| P0 | As of the audit, `Cargo.lock` contains `clap` 4.6.6 from 2026-08-06, the PyO3 0.29.2 family from 2026-08-05, and `thiserror` 2.0.20 from 2026-08-08. The pinned `Swatinem/rust-cache` and `dtolnay/rust-toolchain` commits are also only three to four days old. | Restore pins that cleared the 14-day cool-off and add a fail-closed, tested gate for Cargo, Python, Node, GitHub Actions, and bootstrap tools. |
| P0 | `Index::apply` treats an absolute or parent-escaping operation as `unchanged`; a mixed batch can therefore hide malformed producer input while valid operations mutate state. | Validate the whole observation before mutation and return a typed error without changing any index state. |
| P1 | `IndexHandle::read` returns `std::sync::RwLockReadGuard`, exposing the lock implementation and letting callers hold it across arbitrary work. The public surface also duplicates module and root re-exports and forces a `Vec` allocation for child iteration. | Define the minimal supported API, keep locking inside the abstraction, and use borrowed iteration where ownership permits it. |
| P1 | Running rustdoc with `-D missing-docs` reports 61 undocumented public fields, variants, and methods. Clippy identifies 45 `must_use_candidate` sites while the workspace disables that lint globally. | Reduce the public surface first, document every remaining contract, and apply `#[must_use]` to values whose loss can break correctness rather than enabling the lint indiscriminately. |
| P1 | The index has extensive example tests but no generated reference-model comparison for arbitrary upsert, remove, kind-change, invalidation, and delayed-conditional sequences. | Compare every generated transition against a simple recomputed model with fixed seeds and useful failing traces. |
| P1 | Snapshot tests cover targeted corruption and concurrent writers, but not a broad byte-mutation corpus, concurrent reader visibility, or injected create/write/sync/rename failures. The duplicated FNV-style hash uses `0x1000_0000_01b3`, not the standard FNV-1a 64-bit prime, and has no known-vector test. | Test the parser and commit state machine under arbitrary corruptions and injected failures; name and test the stable fingerprint algorithm. |
| P1 | There is no `rust-toolchain.toml`; normal CI asks for moving `stable`. MSRV is compile-checked but not test-run, and the supported `watch`-without-`cli` feature combination is not in the handoff gate. | Pin a reviewed normal toolchain separately from MSRV and add question-driven feature/MSRV jobs. |
| P2 | Human rendering, JSON truncation detection, and JSON tree rendering recurse once per directory. A sufficiently deep retained tree with a large `--depth` can exhaust the process stack. | Replace recursive rendering walks with explicit stacks and add a deep synthetic-tree subprocess regression. |
| P2 | Extension tallying calls `OsStr::to_str()` and silently omits a non-Unicode filename even when its extension is ASCII. The Python API accepts `&str` paths and emits lossy child/change paths. | Carry native path identity through classification and Python, with reversible platform-specific tests. |
| P2 | `cargo package --list -p fdu` omits the license file. The repository has not yet declared its complete supported-platform, MSRV-change, deprecation, security-reporting, and packaged-artifact contracts. | Make package contents and release policies explicit and smoke-test the packaged artifacts before publishing. |

### Guideline Applicability Review

| Guideline | Disposition for fdu |
| --- | --- |
| `rust-rules.md` | **Apply selectively.** Ownership, domain types, typed errors, module responsibility, safe Rust, and measured-performance rules are already strong. The malformed-observation, lock-guard, public-surface, documentation, `must_use`, and stable-fingerprint findings remain. |
| `rust-project-setup.md` | **Apply.** The two-crate shape, feature boundaries, lint policy, lockfiles, local gate, audits, and pinned action syntax are sound. Toolchain reproducibility, cool-off enforcement, least-privilege workflow settings, trusted-only cache writes, and the feature/MSRV matrix need work. |
| `rust-cli-rules.md` | **Mostly met.** The binary is thin, streams and exits are tested, redirected output is deterministic, broken pipes are quiet success, and golden tests own help/errors/cache behavior. Deep-tree stack safety remains; prompts, destructive dry-run, configuration files, paging, and completions are not current features. |
| `rust-filesystem-rules.md` | **Apply to scan and cache boundaries.** Native path storage, non-following symlink policy, root boundaries, partial-error reporting, private temporary files, atomic replacement, and fail-closed parsing are present. Classification/Python path narrowing and injected snapshot failure-state tests remain. fdu does not mutate the scanned user tree. |
| `rust-testing-rules.md` | **Apply.** Test placement, isolated roots, exact goldens, cross-platform CI, doctests, failure cases, and zero ignored tests are strong. A reference model, broad corrupt-input coverage, injected commit failures, MSRV tests, watch-only features, and minimum-Python wheel coverage add distinct evidence rather than duplicate assertions. |
| `rust-release-rules.md` | **Prepare now, execute later.** No artifact is published, so channels and release credentials remain Phase 1 work. Package contents, one release identity, least privilege, compatibility policy, native artifact smoke tests, and incident/security documentation must be acceptance criteria for the existing publishing bead. |
| `rust-code-review-rules.md` | **Applied by this review.** Automated gates ran first; the review then followed unsafe, data integrity, errors, public API, concurrency, dependencies, performance, tests, and documentation risk order. There is no handwritten unsafe code or FFI pointer manipulation to audit. |
| `porting-principles-and-antipatterns.md` | **Process lessons only.** fdu has no source implementation against which to claim parity. Its useful general rules already apply: tests run in CI, missing tools fail, goldens do not truncate discrepancies, no ignores hide gaps, and defects receive red-before-green tests. Dynamic cross-language corpus parity is not applicable. |
| `python-to-rust-porting-rules.md` | **Not applicable as a port.** `fdu-py` is a binding to the same Rust engine, not an independently translated Python implementation. Its FFI/path boundary is covered by the general Rust, filesystem, testing, and release rules instead. |
| `python-to-rust-cli-porting.md` | **Not applicable.** There is no Python CLI contract to preserve. The native fdu CLI contract is specified directly by the completed golden sessions. |
| `filesystem-heavy-cli-porting.md` | **Not applicable as parity guidance.** No source CLI exists and fdu does not rename, replace, back up, or delete user-tree files. Snapshot cache mutation remains governed by `rust-filesystem-rules.md`. |
| `test-coverage-for-porting.md` | **Not applicable as cross-language mapping.** There is no source test inventory to map. Its fixture-provenance, exact expected-output, surface-enumeration, and discrepancy-classification principles are already reflected in the CLI golden plan and this plan’s model tests. |

## Design

### Approach

Use three gates, in order:

1. **Trust the inputs.** Restore aged dependency and action pins, enforce provenance and
   cool-off, pin the normal Rust toolchain, and make CI permissions and cache behavior
   explicit.
2. **Harden the contracts before optimizing them.** Make observation rejection fallible,
   shrink the public API, hide locks, and add model and persistence tests before packed
   storage and reducers change internal representation.
3. **Finish boundary resilience.** Make deep CLI rendering iterative, preserve native
   paths through classification and Python, then finish package and release evidence.

Every behavior change starts with a discriminating test.
Fixes must investigate the class of failure represented by the first example: one
invalid path implies all invalid component forms; one non-Unicode extension implies Unix
bytes and Windows wide strings; one injected rename failure implies every snapshot
commit stage.

### Components

#### Trust and Reproducibility

- A small, tested cool-off validator owns exact executable-dependency provenance and
  narrow exceptions. It must fail closed when a release date, action commit, checksum, or
  exception field cannot be verified.
- Pull-request validation has top-level read-only permissions, no persisted checkout
  token, and no reusable cache write.
  Trusted branches may write a cache only when the measured benefit justifies it.
- `rust-toolchain.toml` pins the normal compiler and components.
  `rust-version` remains the older compatibility floor, and CI proves both rather than
  conflating them.

#### State and API Contracts

- Observation validation occurs once, before mutation.
  Invalid path syntax rejects the complete batch, leaves the clock and index unchanged,
  and reports the exact offending path.
- Public synchronization APIs return plain data or operation results, not lock guards.
  The final shape must support the actual CLI, Python, watcher, and intended server
  consumers without promising the current `std::sync::RwLock` implementation.
- Supported public items have complete rustdoc and deliberate `must_use` behavior.
  Internal modules and helpers become `pub(crate)` rather than receiving documentation
  solely to satisfy a lint.

#### Refactor Safety Nets

- A simple reference tree recomputes roll-ups from canonical state after generated
  mutations. The implementation and model are compared after every step, not only at the
  end of a sequence.
- Persistence tests separate parse, stage, commit, and cleanup outcomes.
  Atomic visibility is required; crash durability is promised only if parent-directory
  sync is implemented and tested on the supported platforms.
- Fingerprints use one named helper and known vectors.
  Correcting its algorithm may invalidate pre-release cache paths and snapshots; it must
  never reinterpret an old snapshot as current.

#### Product and Language Boundaries

- CLI tree walks use explicit frames so depth consumes heap under a documented bound,
  not call stack. Exact current output remains protected by the golden sessions.
- Classification operates on `OsStr`/native units for path predicates and converts only
  recognized ASCII rule keys to the existing stable string representation.
- Python accepts normal `os.PathLike` values without narrowing at the Rust boundary,
  exposes reversible path identity for children and change records, maps I/O errors to
  useful `OSError` fields, and tests the minimum and current supported Python versions
  from installed wheels.

### API Changes

The exact signatures are settled under red tests, but the intended compatibility
direction is fixed:

- `Index::apply` becomes fallible, or observations become validated values before they
  can reach it; malformed batches cannot return ordinary `ApplyOutcome`
- `IndexHandle::read` no longer exposes `RwLockReadGuard`
- Child iteration on an owned `Index` does not require allocating a `Vec`; shared-handle
  queries may return owned records when that is required to release a lock
- Critical state/feed results are `#[must_use]`
- Classification accepts filesystem-native names
- Python path inputs and outputs are reversible on every supported platform

These are pre-release changes.
No compatibility shim is retained for an unpublished API unless an external consumer is
found during the usage inventory.

### Relationship to Existing Phase 1 Work

| Existing bead | How this plan changes its acceptance criteria |
| --- | --- |
| `fdu-r27g` | Measure contention only after the public API no longer exposes a lock guard; the internal lock remains replaceable. |
| `fdu-1gbl` and `fdu-a6dz` | Packed records and reducers land behind the reference-model suite. Aggregate overflow policy remains owned by `fdu-a6dz`. |
| `fdu-xihx` | The block format must pass the reusable corrupt-input and commit-failure state-machine tests. |
| `fdu-lka2` | Watch queue/backend work must also make timeout distinct from permanent worker disconnection; a stopped watcher cannot look like a quiet tree. |
| `fdu-oqoy` and `fdu-jej9` | Human, JSON, and future JSONL views retain exact current output while moving to stack-safe traversal. The existing raw error-path and schema work remains. |
| `fdu-v4lc` | The rule dialect starts from native path units and cannot reintroduce UTF-8-only extension/basename matching. |
| `fdu-9cf0` | Publishing is blocked on the minimal documented API, complete package contents, support/security policy, cool-off-clean release tooling, and installed-artifact smoke tests. |

## Implementation Plan

### Phase 0: Close the PR Merge Blockers

- [ ] `fdu-ad45`: restore and enforce the 14-day executable-dependency cool-off
- [ ] `fdu-nlh8`: reject malformed observation batches before any mutation
- [ ] `fdu-sn43`: rerun all gates and publish the superseding senior approval after both
  fixes land

The two implementation bugs are independent and should be fixed in parallel when
capacity permits. The final approval bead is deliberately blocked on both.

### Phase 1: Reproducible Tooling and Public Contracts

- [ ] `fdu-zga3`: pin the normal Rust toolchain and complete the feature/MSRV matrix
- [ ] `fdu-s7wr`: seal and document the public Rust API without leaking lock guards

### Phase 2: Add Refactor Safety Nets

- [ ] `fdu-o8r8`: add a deterministic index/delta reference model
- [ ] `fdu-471a`: add snapshot parser and commit-state fault tests with stable
  fingerprint vectors

### Phase 3: Harden Product and Distribution Boundaries

- [ ] `fdu-zsdy`: make human and JSON rendering iterative and stack-safe
- [ ] `fdu-k8zw`: preserve native filesystem identity through classification and Python
  bindings
- [ ] Complete package, compatibility, security, and artifact-smoke acceptance under the
  existing publishing bead `fdu-9cf0`

The bead IDs and dependency graph are recorded in the **Beads** section.

## Testing Strategy

- Keep `make check` as the local handoff gate and make every new required test reachable
  from it
- Run default, all-feature, no-default-feature, and watch-only library combinations for
  the questions each combination uniquely answers
- Run the supported core contract on the exact MSRV and all normal checks on the pinned
  development toolchain
- Use fixed-seed generated operation sequences with the seed and operation trace printed
  on failure; retain minimized discoveries as focused regressions
- Mutate committed snapshot seeds and inject every write/commit failure stage without
  allocating from untrusted declared lengths
- Exercise deep CLI rendering in a child process so a regression is reported as a test
  failure rather than taking down the test runner
- Keep exact tryscript output unchanged unless a separately reviewed product change
  intentionally updates the CLI contract
- Test Unix invalid bytes and Windows invalid wide strings where the platform supports
  them; smoke installed Python wheels at the declared minimum and current versions
- Keep coverage a discovery tool.
  If `cargo-llvm-cov` is adopted, pin it under the cool-off policy and use uncovered
  risk branches to create tests; do not add a percentage target with no behavioral
  rationale

## Rollout Plan

This is pre-release hardening, so breaking internal and unpublished Rust/Python API
changes land before the first crates.io or PyPI release.
Work rolls out in dependency order:

1. repair supply-chain inputs and make observation application atomic as independent P0
   fixes;
2. rerun the complete PR gate and publish final approval only after both pass;
3. pin and prove toolchain/feature contracts;
4. shrink and document the public API under red tests;
5. land the index model and snapshot state-machine safety nets;
6. harden CLI and language boundaries;
7. let the existing Phase 1 engine and publishing beads consume those gates.

No migration is needed for pre-release snapshots.
A fingerprint or format correction must cause a cold scan, never an attempted
reinterpretation.

## Open Questions

- **Guard-free shared-index API (`fdu-s7wr`)**: which query shape best serves the first
  real concurrent consumer—focused owned methods, a bounded immutable view, or another
  measured design? Returning a standard-library lock guard is not an option.
- **Pinned normal Rust release (`fdu-zga3`)**: which exact release has cleared the
  14-day cool-off when the toolchain bead is implemented?
  The plan pins that reviewed release; it does not encode today’s moving `stable`
  answer.
- **Snapshot durability contract (`fdu-471a`)**: is crash durability a product
  requirement, or is atomic visibility plus disposable-cache recovery sufficient?
  The current architecture needs only the latter; stronger fsync claims require platform
  evidence.

## Beads

Epic: **fdu-dxee** — Harden fdu against the Rust engineering quality audit.
It is a child of the Phase 1 epic `fdu-qfz6` so the urgent fixes and later refactor
guards appear in one program graph.

Audit and planning record: **fdu-qcgm** — Review the playbook, capture evidence, write
this plan, and assemble the non-duplicative implementation graph.
That bead is closed; the implementation beads remain open.

| Phase | Bead | Priority | Work | Direct blocker |
| --- | --- | --- | --- | --- |
| 0 | `fdu-ad45` | P0 | Restore and enforce the 14-day executable-dependency cool-off | — |
| 0 | `fdu-nlh8` | P0 | Reject malformed observation batches before mutation | — |
| 1 | `fdu-zga3` | P1 | Pin Rust tooling and prove supported feature/MSRV contracts | `fdu-ad45`, `fdu-sn43` |
| 1 | `fdu-s7wr` | P1 | Seal the minimal guard-free Rust API | `fdu-nlh8`, `fdu-sn43` |
| 2 | `fdu-o8r8` | P1 | Add a deterministic index/delta reference model | `fdu-nlh8`, `fdu-sn43` |
| 2 | `fdu-471a` | P1 | Exercise snapshot parsing and commit failures as a state machine | `fdu-nlh8`, `fdu-sn43` |
| 3 | `fdu-zsdy` | P2 | Make CLI rendering iterative and stack-safe | `fdu-sn43` |
| 3 | `fdu-k8zw` | P2 | Preserve native identity through classification and Python | `fdu-s7wr` |

The Phase 1 bead `fdu-sn43` depends on both P0 bugs and owns final PR validation and
approval. This keeps the merge decision explicit without making the two unrelated fixes
block each other.

Cross-epic dependencies make the existing work consume these gates:

- `fdu-ad45` blocks `fdu-zga3`, comparator acquisition under `fdu-k5t5`, publishing
  under `fdu-9cf0`, and final approval under `fdu-sn43`.
- `fdu-nlh8` blocks `fdu-s7wr`, `fdu-o8r8`, `fdu-471a`, and final approval under
  `fdu-sn43`.
- `fdu-sn43` is the explicit post-approval start gate for `fdu-zga3`, `fdu-s7wr`,
  `fdu-o8r8`, `fdu-471a`, and `fdu-zsdy`.
- `fdu-s7wr` blocks `fdu-r27g`, `fdu-1gbl`, `fdu-a6dz`, `fdu-lka2`, and `fdu-9cf0`.
- `fdu-o8r8` blocks `fdu-1gbl` and `fdu-a6dz`.
- `fdu-471a` blocks `fdu-xihx`.
- `fdu-zga3` blocks `fdu-ywu0` and `fdu-9cf0`.
- `fdu-zsdy` blocks `fdu-oqoy` and `fdu-jej9`.
- `fdu-k8zw` blocks `fdu-jej9`, `fdu-v4lc`, and `fdu-9cf0`.

The supply-chain bead also owns the tbd integration drift reported during this review.
`tbd doctor` marks the managed `AGENTS.md` and Codex hook surfaces stale, while a dry
run would remove two legacy hooks.
Any refresh must be reviewed as an open-time executable change and preserve fdu’s
explicit opt-in-only GitHub CLI bootstrap.

## References

- [Rust Porting Playbook guideline index at the reviewed commit](https://github.com/jlevy/rust-porting-playbook/blob/d24760a3fbd2951c730a199269aeb082abb46a42/guidelines/README.md)
- [fdu Phase 1 plan](plan-2026-08-08-fdu-phase-1.md)
- [Completed CLI golden-test plan](../done/plan-2026-08-09-fdu-cli-golden-tests.md)
- [File roll-up engine research](../../research/research-2026-08-06-file-rollup-engine.md)
- [`AGENTS.md`](../../../../AGENTS.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
