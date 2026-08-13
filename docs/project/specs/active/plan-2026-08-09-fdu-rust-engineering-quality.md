# Feature: Rust Engineering Quality Hardening

**Date:** 2026-08-09

**Author:** fdu project

**Status:** Active — PR #1 merged; CLI stack hardening in follow-up validation

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

The epic is `fdu-dxee`, a child of the Phase 1 epic `fdu-qfz6`. Every design,
implementation, and validation node on the PR #1 merge path is closed and merged:

- `fdu-ad45`: executable-dependency cool-off, provenance, and CI trust controls now fail
  closed in CI and the local handoff gate;
- `fdu-nlh8`: whole-batch path validation happens before any index mutation;
- `fdu-s7wr`: the shared-index surface returns owned values and never exposes a lock
  guard or receiver;
- `fdu-1j0b`: watch verification and reconciliation perform filesystem I/O after index
  locks are released;
- `fdu-8jte`: all watcher stages are bounded, overload degrades to a sticky root
  invalidation, and cancellation joins the worker without blocking sends;
- `fdu-gd6n`: deterministic cross-thread tests prove writer ordering, whole-batch reader
  visibility, freshness epochs, typed failure paths, old-or-new snapshots, watcher
  ownership, and the Python GIL/borrow contract.
- `fdu-l8vc`: the unsupported rootless public watch-apply helper is test-only; the
  supported applying driver proves watcher/index root identity before consuming work;
- `fdu-83gl`: the watch contract now distinguishes the filesystem `stat` sample point
  from in-memory writer arbitration and documents queued-event convergence;
- `fdu-ie5z`: terminal-clock no-op and stale observations report their arbitration
  result, while a real mutation still fails atomically.
- `fdu-b3qe`: GitHub provenance checks use an explicit workflow token in CI and a
  non-shell local `gh` credential fallback, while retaining read-only permissions and
  fail-closed validation.
- `fdu-9xf7`: the Windows-only missing-doc failure is corrected by retaining crate
  documentation before the Unix cfg attribute; exact Windows-target compilation and the
  complete local gate pass, and fresh cross-platform CI confirms the correction.

`fdu-zga3` also completed early because reproducible review evidence required the pinned
normal toolchain, watch-only feature lane, and test-running MSRV lane now rather than
after merge.

The final merge-path bead `fdu-sn43` is closed.
The original merge gate passed 145 all-feature library tests, two CLI unit tests, one
CLI integration test, two doctests, and 25 built-binary golden scenarios.
The focused CLI follow-up now passes 148 all-feature library tests, four CLI process
tests, two doctests, 26 golden scenarios, 105 core-only tests, 135 watch-only tests,
exact-1.85.0 compilation and core tests, ten live supply-chain policy tests, Clippy,
rustdoc, Cargo/npm audits, two Python concurrency tests, installed abi3
wheel/module/console smoke, and direct local-wheel `uvx` execution.
Fresh [GitHub run 31339731585](https://github.com/jlevy/fdu/actions/runs/31339731585)
passes the Linux, macOS, Windows, MSRV, feature-boundary, docs, audit, provenance,
golden, and Python jobs, and the superseding senior review approved the revision that
merged through PR #1. The remaining P1 and P2 items protect later representation
changes, performance evidence, and publishing; they are explicitly deferred rather than
hidden merge blockers.

Python 3.12 is now the minimum for the unpublished wheel and repository-owned Python
tooling. The wheel uses `abi3-py312`; locked tests run on 3.12, and CI builds and smokes
the same abi3 artifact on 3.12 and 3.14 across Linux, macOS, and Windows.
The first-party `jlevy/simple-modern-uv` v0.4.0 template was used as a conventions
source without applying its pure-Python project structure to this Cargo workspace and
maturin extension.

## Goals

- Restore the repository’s stated 14-day executable-dependency cool-off and enforce it
  in the handoff gate
- Make malformed producer input fail before any index mutation instead of looking like
  an unchanged observation
- Keep synchronization primitives and unnecessary allocations out of the supported Rust
  API
- Keep filesystem I/O, blocking sends, serialization, Python conversion, and user
  callbacks outside index locks
- Bound every watcher pipeline stage and turn overload into explicit reconciliation,
  with cancellable joined worker ownership
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
| P1 | After three optimistic clock conflicts, `watch::apply_reverified` takes the index writer lock and calls `symlink_metadata` once per queued operation before releasing it. A slow filesystem or large observation can block all readers and writers indefinitely. | Introduce a linearizable apply-if-clock operation; perform every stat outside the lock, then escalate exhausted conflicts to an explicit root invalidation and ordinary reconciliation rather than spinning or moving I/O under the lock. |
| P1 | The raw notify channel, verified-observation channel, and pending-path map are unbounded. Replacing either channel with a blocking bounded send would let a full output queue deadlock `Watcher::drop` while it joins the worker and still owns the receiver. The worker also performs stats that the applying path repeats. | Make the joined worker an I/O-free bounded coalescer, verify on the consuming thread immediately before arbitration, and use nonblocking ingress/output, capped state, sticky overflow-to-root invalidation, explicit cancellation, and stop distinct from timeout. |
| P1 | Existing shared-handle tests are sequential and do not prove batch visibility, writer linearization, freshness epochs under real overlap, snapshot old-or-new visibility, worker teardown, or Python GIL/same-object behavior. | Add deterministic state-machine tests using barriers, channels, injectable seams, and bounded deadlines; require a model checker before adopting future custom lock-free protocols. |
| P0 | The free public watch observation helper carried only relative paths, so it could not prove that an observation and `IndexHandle` represented the same root. | Keep unrooted observations generic and make the root-checked `Watcher::apply_next` driver the only supported watch-application boundary. |
| P0 | `Index::apply` checked for the next logical clock before arbitration, so an entirely unchanged or stale nonempty observation failed at the terminal clock even though it committed nothing. | Probe the otherwise infallible mutation phase only on the unreachable terminal-clock path; return no-op/stale stats without mutation and reject a real change atomically. |
| P0 | The live provenance gate depended on GitHub’s shared unauthenticated 60-request quota locally and failed with HTTP 403 after that quota was exhausted. | Prefer explicit workflow/environment tokens, fall back to the authenticated local `gh` credential without a shell or output, and test that CI supplies only its read-only workflow token. |
| P0 | Windows compiled the Unix-only CLI exit integration target as an empty crate; placing `#![cfg(unix)]` before its `//!` documentation removed the docs before workspace `missing_docs` enforcement. | Keep crate documentation before the cfg attribute, compile-check the exact Windows target locally with warnings denied, and require a fresh Windows CI pass. |
| P1 | An automated review treated the interval between filesystem `stat` and index commit as lockable, which would encourage lock-held I/O or repeated stats without eliminating the external race. | Specify the attainable contract: the sample is valid at its `stat` point, later backend events remain queued, loss invalidates and reconciles, and the clock boundary arbitrates only in-memory writers. |
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
| `rust-rules.md` | **Apply selectively.** Ownership, domain types, typed errors, module responsibility, safe Rust, and measured-performance rules are already strong. The malformed-observation, lock lifetime, bounded-worker-lifecycle, public-surface, documentation, `must_use`, and stable-fingerprint findings remain. |
| `rust-project-setup.md` | **Applied.** The two-crate shape, feature boundaries, lint policy, lockfiles, local gate, and audits were already sound. `fdu-ad45` and `fdu-zga3` now provide toolchain reproducibility, cool-off enforcement, least privilege, cache-free pull-request jobs, and the feature/MSRV matrix. |
| `rust-cli-rules.md` | **Mostly met.** The binary is thin, streams and exits are tested, redirected output is deterministic, broken pipes are quiet success, golden tests own help/errors/cache behavior, and explicit-stack renderers cover deep trees. Prompts, destructive dry-run, configuration files, paging, and completions are not current features. |
| `rust-filesystem-rules.md` | **Apply to scan and cache boundaries.** Native path storage, non-following symlink policy, root boundaries, partial-error reporting, private temporary files, atomic replacement, and fail-closed parsing are present. Classification/Python path narrowing and injected snapshot failure-state tests remain. fdu does not mutate the scanned user tree. |
| `rust-testing-rules.md` | **Apply selectively.** Test placement, isolated roots, exact goldens, cross-platform CI, doctests, failure cases, deterministic concurrency, MSRV tests, and watch-only coverage are strong. A reference model, broad corrupt-input coverage, injected commit failures, and minimum-Python wheel coverage remain distinct later evidence. |
| `rust-release-rules.md` | **Prepare now, execute later.** No artifact is published, so channels and release credentials remain Phase 1 work. Package contents, one release identity, least privilege, compatibility policy, native artifact smoke tests, and incident/security documentation must be acceptance criteria for the existing publishing bead. |
| `rust-code-review-rules.md` | **Applied by this review.** Automated gates ran first; the review then followed unsafe, data integrity, errors, public API, concurrency, dependencies, performance, tests, and documentation risk order. There is no handwritten unsafe code or FFI pointer manipulation to audit. |
| `porting-principles-and-antipatterns.md` | **Process lessons only.** fdu has no source implementation against which to claim parity. Its useful general rules already apply: tests run in CI, missing tools fail, goldens do not truncate discrepancies, no ignores hide gaps, and defects receive red-before-green tests. Dynamic cross-language corpus parity is not applicable. |
| `python-to-rust-porting-rules.md` | **Not applicable as a port.** `fdu-py` is a binding to the same Rust engine, not an independently translated Python implementation. Its FFI/path boundary is covered by the general Rust, filesystem, testing, and release rules instead. |
| `python-to-rust-cli-porting.md` | **Not applicable.** There is no Python CLI contract to preserve. The native fdu CLI contract is specified directly by the completed golden sessions. |
| `filesystem-heavy-cli-porting.md` | **Not applicable as parity guidance.** No source CLI exists and fdu does not rename, replace, back up, or delete user-tree files. Snapshot cache mutation remains governed by `rust-filesystem-rules.md`. |
| `test-coverage-for-porting.md` | **Not applicable as cross-language mapping.** There is no source test inventory to map. Its fixture-provenance, exact expected-output, surface-enumeration, and discrepancy-classification principles are already reflected in the CLI golden plan and this plan’s model tests. |

## Design

### Approach

Use four gates, in order:

1. **Trust the inputs.** Restore aged dependency and action pins, enforce provenance and
   cool-off, pin the normal Rust toolchain, and make CI permissions and cache behavior
   explicit.
2. **Make mutation and concurrency fail safe.** Make observation rejection fallible,
   hide lock capabilities, keep I/O and callbacks outside locks, and bound the watcher
   with explicit overload and shutdown semantics.
3. **Prove the contracts before optimizing them.** Exercise real interleavings, then add
   model and persistence tests before packed storage and reducers change internal
   representation.
4. **Finish boundary resilience.** Make deep CLI rendering iterative, preserve native
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

#### Concurrency and Lifecycle Contracts

- One accepted observation has one writer-lock linearization point.
  Concurrent readers see the complete state before or after that point, including
  roll-ups, clock, journal, and freshness; they never see a partially applied batch.
- Watch verification uses a bounded optimistic apply-if-clock loop.
  Exhausted conflicts publish a conservative root invalidation with an explicit
  contention reason and reconcile after unlock; progress never depends on holding a lock
  across `stat` or on an unbounded retry loop.
- A verified watch sample is linearized at its filesystem `stat`, not at a fictional
  filesystem-wide lock.
  Events that race after that sample stay in the bounded backend queue and converge in a
  later batch; reported loss or ambiguity invalidates and reconciles.
  The clock boundary arbitrates in-memory writers and does not claim to make external
  filesystem mutation transactional.
- Index locks protect in-memory state only.
  Filesystem I/O, snapshot serialization and commit, blocking channel operations, Python
  object conversion, and user-provided sinks run after the lock is released.
  The current design has no nested index locks; any future second lock must document and
  test one global acquisition order.
- `IndexHandle` owns synchronization and returns plain owned data or results.
  It does not expose guards, receivers, or arbitrary closures that could retain a
  capability or re-enter a writer operation and self-deadlock.
  Poisoning and worker failure become typed outcomes rather than panics or quiet
  timeouts.
- Watch ingress, pending coalescing, and coalesced output have explicit bounds.
  Backend callbacks never block.
  Capacity loss sets a sticky root `WatchOverflow` invalidation; the system may do extra
  reconciliation but may never silently claim freshness.
- The joined watch worker only coalesces bounded hints; it performs no filesystem I/O.
  `next_observation` or `apply_next` verifies on the consuming thread immediately before
  arbitration, avoiding duplicated stats and keeping worker teardown cancellable.
- Watcher cancellation has one owner, wakes every wait, and joins every worker.
  A full output queue, disconnected consumer, in-flight batch, backend error, or worker
  panic cannot deadlock teardown or leave a detached thread.
- Snapshot replacement gives concurrent readers one complete old or new image.
  Shared index capture releases the index lock before serialization and filesystem
  commit.
- PyO3 native work releases the GIL so unrelated Python threads progress.
  Access to one `PyIndex` follows an explicit serialized-or-rejected borrow contract; it
  is not described as concurrent shared-index reading unless its representation changes.
- The current safe `std::sync::RwLock` design needs deterministic interleaving tests,
  not a model-checker dependency.
  Any custom atomic-refcount, work-stealing, unsafe, or lock-free Phase 1 protocol must
  specify memory orderings and pass a small model-checking harness before adoption.
  `unsafe_code = "deny"` remains the default; an intrusive unsafe queue requires
  measured need, a separate reviewed boundary, written invariants, and Miri where
  applicable.

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

#### Shared-Index Consumer Inventory

| Consumer | Required ownership surface | Lock boundary |
| --- | --- | --- |
| CLI renderer | Owned `Index`; borrowed child iterators and entry fields | No shared lock |
| Python binding | Owned `Index` inside one `PyIndex` | PyO3 rejects overlapping access while `refresh()` holds its exclusive borrow; native open/scan/refresh work releases the GIL |
| Direct scan/reconcile | Mutable owned `Index` | No shared lock |
| Applying reconcile | `IndexHandle` expectations, child-state capture, apply, and freshness transitions | Each focused operation acquires and releases internally; sinks run afterward |
| Watch driver | `IndexHandle` root/scope/clock capture, apply-if-clock, and root invalidation | Verification happens before the writer lock; reconciliation and sinks happen after it |
| Snapshot writer | Owned `Index`, or `IndexHandle::snapshot()` followed by `snapshot::save()` | A coherent clone is captured under a read lock; encoding and filesystem commit happen after release |
| Future server/UI | Owned totals, roll-ups, metadata, child snapshots, history, or coherent full snapshots | No guard or receiver is part of the supported API |

No current consumer needs a public `RwLock` guard, writer guard, channel receiver, or
lock-held callback. The `index` and `types` implementation modules are private; their
supported types are exported once at the crate root.

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
| `fdu-r27g` | Retain the current standard-library lock and measure contention only after the public API no longer exposes a guard; change primitives only from evidence. |
| `fdu-gdrv` and `fdu-aky1` | Specify ownership, memory orderings, bounded work, cancellation, panic propagation, and joins. Begin with safe scoped workers/queues; model-check custom atomics and require separate evidence before relaxing the unsafe-code ban. |
| `fdu-1gbl` and `fdu-a6dz` | Packed records and reducers land behind the reference-model suite. Aggregate overflow policy remains owned by `fdu-a6dz`. |
| `fdu-xihx` | The block format must pass the reusable corrupt-input and commit-failure state-machine tests. |
| `fdu-lka2` | Platform rename, backend selection, descriptor, and failed-coverage work consumes the bounded transport from `fdu-8jte`; it does not reopen generic queue or shutdown semantics. |
| `fdu-oqoy` and `fdu-jej9` | Human, JSON, and future JSONL views retain exact current output while moving to stack-safe traversal. The existing raw error-path and schema work remains. |
| `fdu-v4lc` | The rule dialect starts from native path units and cannot reintroduce UTF-8-only extension/basename matching. |
| `fdu-9cf0` | Publishing is blocked on the minimal documented API, complete package contents, support/security policy, cool-off-clean release tooling, and installed-artifact smoke tests. |

## Implementation Plan

### Phase 0: Close the PR Merge Blockers

- [x] `fdu-ad45`: restore and enforce the 14-day executable-dependency cool-off
- [x] `fdu-nlh8`: reject malformed observation batches before any mutation
- [x] `fdu-1j0b`: remove filesystem verification from the writer-lock fallback
- [x] `fdu-8jte`: make the worker an I/O-free bounded coalescer and make overload and
  shutdown fail safe
- [x] `fdu-s7wr`: seal the shared-index API without public guards or lock-held callbacks
- [x] `fdu-gd6n`: prove the combined concurrency contract under deterministic
  interleavings
- [x] `fdu-l8vc`: remove the unrooted public watch-application capability
- [x] `fdu-83gl`: specify the stat-sample and queued-event convergence contract
- [x] `fdu-ie5z`: preserve no-op and stale arbitration at the terminal logical clock
- [x] `fdu-b3qe`: authenticate live provenance checks without widening PR permissions
- [x] `fdu-9xf7`: confirm the cfg-disabled integration-test documentation fix in fresh
  Windows CI
- [x] `fdu-sn43`: rerun all gates and publish the superseding senior approval

The supply-chain, watch-lock, and watch-transport fixes are independent.
Atomic batch rejection precedes the final guard-free API because it changes the apply
result contract. The deterministic concurrency suite follows all four state/lifecycle
corrections, then final approval follows that suite and the independent supply-chain
fix.

### Phase 1: Reproducible Tooling

- [x] `fdu-zga3`: pin the normal Rust toolchain and complete the feature/MSRV matrix

### Phase 2: Add Refactor Safety Nets

- [ ] `fdu-o8r8`: add a deterministic index/delta reference model
- [ ] `fdu-471a`: add snapshot parser and commit-state fault tests with stable
  fingerprint vectors

### Phase 3: Harden Product and Distribution Boundaries

- [x] `fdu-zsdy`: make human and JSON rendering iterative and stack-safe; the focused
  CLI follow-up preserves exact output and adds a small-stack subprocess regression
- [x] `fdu-k8zw`: preserve native filesystem identity through classification and Python
  bindings
- [x] `fdu-c7z2`: raise the wheel and tooling minimum to Python 3.12 and align PyO3, uv,
  CI, and documentation
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
- Use barriers and handoff channels to place threads at exact transitions.
  Every wait has a bounded deadline that fails with the captured state; synchronization
  never depends on a scheduling sleep
- Prove simultaneous writers receive unique contiguous clocks, readers observe whole
  batches, newer invalidations survive older reconciliation completion, and reentrant
  sinks run after unlock
- Fill every watcher stage, disconnect every owner, and drop during queued and in-flight
  work; prove bounded state, root-invalidation degradation, typed stop/panic outcomes,
  no duplicated verification, and prompt worker join
- Race snapshot readers and writers; exercise Python GIL release and same-object borrow
  behavior in the locked embedding lane, then smoke the installed wheel separately
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

1. **Complete:** repair supply-chain inputs and make observation application atomic as
   independent P0 fixes;
2. **Complete:** remove watch I/O from index locks, bound watcher lifecycle, and seal
   the shared-index API under red tests;
3. **Complete:** pass the deterministic concurrency state-machine suite;
4. **Complete:** rerun the complete PR gate and publish final approval;
5. **Complete early:** pin and prove toolchain/feature contracts;
6. land the index model and snapshot failure-state safety nets;
7. harden CLI and language boundaries;
8. let the existing Phase 1 engine and publishing beads consume those gates.

No migration is needed for pre-release snapshots.
A fingerprint or format correction must cause a cold scan, never an attempted
reinterpretation.

## Resolved and Open Questions

- **Guard-free shared-index API (`fdu-s7wr`) — resolved:** focused owned methods serve
  shared consumers, `IndexHandle::snapshot()` captures a coherent owned image, and the
  owned `Index` retains allocation-free borrowed child iteration.
  No supported API exposes a standard-library lock guard.
- **Pinned normal Rust release (`fdu-zga3`) — resolved:** 1.97.1 and its rustfmt/Clippy
  components are pinned after provenance review.
  Core-only and watch-only tests run locally and in CI; the exact 1.85.0 MSRV lane
  compiles all features and runs the core tests rather than relying on compilation
  alone.
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
The focused concurrency review is **fdu-07t4**. The playbook audit is closed; the
concurrency review closes when this plan, its beads, and the PR record agree.

| Phase | Bead | Priority | Work | Direct blockers |
| --- | --- | --- | --- | --- |
| 0 | `fdu-ad45` | P0 | Restore and enforce the 14-day executable-dependency cool-off | — |
| 0 | `fdu-nlh8` | P0 | Reject malformed observation batches before mutation | — |
| 0 | `fdu-1j0b` | P1 | Keep watch arbitration free of filesystem I/O under index locks | — |
| 0 | `fdu-8jte` | P1 | Bound watcher transport and make overload/shutdown fail safe | — |
| 0 | `fdu-s7wr` | P1 | Seal the minimal guard-free Rust API and ownership contract | `fdu-nlh8` |
| 0 | `fdu-gd6n` | P1 | Prove current concurrency contracts deterministically | `fdu-s7wr`, `fdu-1j0b`, `fdu-8jte` |
| 0 | `fdu-l8vc` | P0 | Bind supported watch application to the indexed root | — |
| 0 | `fdu-83gl` | P0 | Specify watch stat-to-commit linearization and convergence | — |
| 0 | `fdu-ie5z` | P0 | Allow no-op and stale observations at the terminal clock | — |
| 0 | `fdu-b3qe` | P0 | Authenticate live provenance checks with least privilege | — |
| 0 | `fdu-9xf7` | P0 | Keep cfg-disabled integration-test crates documented cross-platform | — |
| 1 | `fdu-zga3` | P1 | Pin Rust tooling and prove supported feature/MSRV contracts | `fdu-ad45` |
| 2 | `fdu-o8r8` | P1 | Add a deterministic index/delta reference model | `fdu-nlh8`, `fdu-sn43` |
| 2 | `fdu-471a` | P1 | Exercise snapshot parsing and commit failures as a state machine | `fdu-nlh8`, `fdu-sn43` |
| 3 | `fdu-zsdy` | P2 | Make CLI rendering iterative and stack-safe (complete) | `fdu-sn43` |
| 3 | `fdu-c7z2` | P1 | Raise the supported Python floor to 3.12 and align uv packaging | — |
| 3 | `fdu-k8zw` | P2 | Preserve native identity through classification and Python | `fdu-s7wr` |

The Phase 1 bead `fdu-sn43` depends on `fdu-ad45`, `fdu-gd6n`, `fdu-l8vc`, `fdu-83gl`,
`fdu-ie5z`, `fdu-b3qe`, and `fdu-9xf7` and owns final PR validation and approval.
Atomic rejection reaches it transitively through `fdu-s7wr` and the validation gate,
without serializing independent fixes.

Cross-epic dependencies make the existing work consume these gates:

- `fdu-ad45` blocks `fdu-zga3`, comparator acquisition under `fdu-k5t5`, publishing
  under `fdu-9cf0`, and final approval under `fdu-sn43`.
- `fdu-nlh8` blocks `fdu-s7wr`, `fdu-o8r8`, `fdu-471a`, and final approval transitively
  through `fdu-gd6n` and `fdu-sn43`.
- `fdu-1j0b`, `fdu-8jte`, and `fdu-s7wr` block final concurrency validation under
  `fdu-gd6n`; `fdu-gd6n` blocks `fdu-sn43`.
- `fdu-l8vc`, `fdu-83gl`, and `fdu-ie5z` are final-review corrections that directly
  block `fdu-sn43` without serializing one another.
- `fdu-b3qe` is the final-gate provenance fix and directly blocks `fdu-sn43`.
- `fdu-9xf7` was the cross-platform documentation-lint fix; fresh Windows CI closed it
  before `fdu-sn43` completed.
- `fdu-sn43` is the explicit post-approval start gate for `fdu-o8r8`, `fdu-471a`, and
  `fdu-zsdy`; `fdu-zga3` completed early because the merge gate already needed its
  reproducibility and feature evidence.
- `fdu-s7wr` blocks `fdu-r27g`, `fdu-1gbl`, `fdu-a6dz`, `fdu-lka2`, and `fdu-9cf0`.
- `fdu-o8r8` blocks `fdu-1gbl` and `fdu-a6dz`.
- `fdu-471a` blocks `fdu-xihx`.
- `fdu-zga3` blocks `fdu-ywu0` and `fdu-9cf0`.
- `fdu-zsdy` blocks `fdu-oqoy` and `fdu-jej9`.
- `fdu-c7z2` blocks minimum-version coverage under `fdu-k8zw` and publishing under
  `fdu-9cf0`.
- `fdu-k8zw` blocks `fdu-jej9`, `fdu-v4lc`, and `fdu-9cf0`.

The supply-chain bead also resolved the tbd integration drift reported during this
review. The managed surfaces were refreshed with GitHub CLI auto-bootstrap disabled; the
two legacy session-start installers were removed.
The capability remains as the explicit, checksum-verified `scripts/bootstrap-gh-cli.sh`,
and `tbd doctor` is clean.

## References

- [Rust Porting Playbook guideline index at the reviewed commit](https://github.com/jlevy/rust-porting-playbook/blob/d24760a3fbd2951c730a199269aeb082abb46a42/guidelines/README.md)
- [fdu Phase 1 plan](plan-2026-08-08-fdu-phase-1.md)
- [Completed CLI golden-test plan](../done/plan-2026-08-09-fdu-cli-golden-tests.md)
- [File roll-up engine research](../../research/research-2026-08-06-file-rollup-engine.md)
- [`AGENTS.md`](../../../../AGENTS.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
