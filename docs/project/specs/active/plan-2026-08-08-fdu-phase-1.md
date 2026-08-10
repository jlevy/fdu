# Plan: fdu Phase 1 — Core Engine, CLI, and Benchmarks

**Date:** 2026-08-08

**Status:** Active

**Background:**
[research-2026-08-06-file-rollup-engine.md](../../research/research-2026-08-06-file-rollup-engine.md)
— the twelve-tool survey and the architecture this plan builds.
The detailed benchmark methodology and implementation graph are in
[plan-2026-08-09-fdu-end-to-end-performance-testing.md](plan-2026-08-09-fdu-end-to-end-performance-testing.md).

## Working on This

Phase 0 merged to `main` through [PR #1](https://github.com/jlevy/fdu/pull/1) at merge
commit `92ee5ab`. All P0, concurrency, and final-validation blockers in **Wave 0** below
are closed. The merged revision passed the complete local handoff gate and the fresh
[Linux/macOS/Windows matrix](https://github.com/jlevy/fdu/actions/runs/31339731585). The
focused CLI UX, agent-skill, and wheel-entry-point follow-up is tracked by `fdu-6c8n` on
a new branch from `origin/main`; Phase 1 performance work remains separate and makes no
claim about the portable walker.

Beads live on the `tbd-sync` branch and are visible from any clone (`tbd list`).
`make check` is the handoff gate; `AGENTS.md` carries the conventions worth not
rediscovering.

## Where This Stands

The Phase 0 product slice is implemented: the repository exists, the architecture is
expressed in code, and the whole pipeline runs end to end.
Phase 0 hardening is merged: the local gate, fresh cross-platform matrix, automated
review disposition, and tbd integrity/synchronization checks pass, and the final senior
review records no unresolved blocker.
What that means concretely, because “scaffold” is otherwise an unhelpful word:

**Built and tested.**

| Piece | Module | State |
| --- | --- | --- |
| Observation/commit contract | `types.rs` | Conditional producer observations; only effective accepted ops become clocked `AppliedDelta` |
| In-memory index | `index.rs` | Parent-pointer arena, generation/revision-safe arbitration, per-directory roll-ups, O(depth) apply, bounded feed |
| Roll-up reducers | `index.rs` | Counts, apparent and allocated bytes, pre-epoch-safe newest mtime, per-extension tallies — all hierarchical |
| Walk and reconcile | `scan.rs` | Scope-safe applying full/subtree reconciliation with explicit freshness and bounded producer batches; correct and portable, **not fast** |
| Snapshot | `snapshot.rs` | Flat format v2 bootstrap; bounded streaming load, payload checksum, semantic scope, owner-only concurrent replacement |
| Watch layer | `watch.rs` | notify-backed bounded I/O-free coalescer plus consuming-thread verification and apply/reconcile driver; sticky overflow invalidation, typed stop/panic, and joined cancellation; not started by `open()` or Python |
| CLI | `cli.rs` | Human tree, schema-v2 JSON, exact kinds/errors, partial exit status, and deterministic color/stream behavior; the focused follow-up adds complete help and a portable skill |
| Python bindings | `fdu-py` | Bulk API, retained scan scope, freshness/errors, explicit GIL release and same-object borrow exclusion; the focused follow-up adds the same Rust CLI as the wheel entry point |
| CI | `.github/workflows/ci.yml` | SHA-pinned Actions; locked three-OS tests, MSRV, docs, audit, native goldens, and installed-wheel smoke |

The workspace suite includes adversarial ordering and ABA arbitration, cache-scope,
non-UTF-8 identity, snapshot integrity/resource/permission bounds, concurrent
replacement, invalidation-retry, partial-exit, and installed-wheel tests.
Clippy pedantic is clean with `unsafe_code = "deny"` on the core crate; the
`--no-default-features` library path is built and tested in CI rather than assumed.

**Deliberately not built.** The walker is `read_dir` + `symlink_metadata`. Nothing in
this repository is fast yet, and **no performance claim should be made until the syscall
layer lands and the benchmark gate passes**. The snapshot format is a flat uncompressed
image whose reader is bounded and streaming but whose writer still materializes the
image. `open()` blocks until warm reconciliation finishes.
`IndexHandle` now seals the single-writer implementation behind focused owned queries,
and snapshot serialization, watcher verification, callbacks, and Python conversion run
after its locks are released.
No server or Python watcher integration is implied yet.
Permanent backend-failure marking, rename stitching, and filesystem-specific backend
selection remain in the later watch-hardening bead.
These pieces settle the contract, cache lifecycle, and CI matrix before the syscall and
packed-layout work that is harder to change later.

Findings from implementation and the final branch-wide review, worth recording because
they were not in the research:

- Snapshot parsing now bounds the whole image, count, and path fields before allocation
  and rebuilds records incrementally.
  “Corrupt equals empty” is not a policy you get by intending it; resource bounds and
  sparse/oversized regression tests are part of it.
- The watcher deadlocked on shutdown because the worker thread was joined before the
  notify watcher was dropped, and dropping it is what closes the channel the worker
  waits on. Ordering is now explicit and commented.
  This is the kind of thing a feature flag hides until someone turns it on.
- A verified watch sample is not necessarily current by the time a consumer drains its
  queue. The applying driver now re-stats against a clock-stable index boundary and
  rejects a watcher/index root mismatch before consuming an observation.
  A concurrency review then found that its sustained-conflict fallback performed those
  stats while holding the writer lock; `fdu-1j0b` replaced that fallback with bounded
  apply-if-clock attempts and conservative reconciliation.
- The existing watcher shutdown ordering is correct for unbounded sends, but both
  channels and the pending-path map can grow without limit.
  A naive blocking bound would reintroduce a full-output-queue shutdown deadlock.
  `fdu-8jte` made the joined worker a bounded I/O-free coalescer, moved verification to
  the consuming thread immediately before arbitration, and owns the complete nonblocking
  overflow, cancellation, and join protocol.
- Public `IndexHandle::read` lets callers hold a read lock across arbitrary work or
  self-deadlock by applying while the guard remains live.
  `fdu-s7wr` sealed that API, and `fdu-gd6n` proves the resulting ownership and
  visibility contract with real interleavings.
- Operational batch size is still untrusted allocation input.
  Zero and oversized values now fail before allocation, and filesystem-boundary mode
  fails explicitly where the platform cannot supply device identity.
- The Python artifact had accidentally enabled the optional watch feature despite
  exposing no watcher.
  The wheel now proves the watch layer is deletable, and Windows cache discovery uses
  its native local-app-data location.
- Zero cannot be both the max-reducer identity and a valid epoch timestamp.
  Newest-mtime reduction now uses file presence as its identity and preserves negative
  timestamps.

## What Phase 1 Delivers

Goal 1 met and *demonstrated*: the fastest walker available that also returns full
detailed stats, with a cache that makes warm runs near-instant, benchmarked honestly
against dut and gdu.
Plus the CLI as a finished product surface, and the type-rule dialect defined early
enough that plugins never need two rule languages.

Phase 1 explicitly excludes content-tier metrics (words, sentences, paragraphs) and a
durable cross-restart delta journal.
A bounded process-local `AppliedDelta` feed exists; persisting or compacting it remains
separate work until the stat tier is solid.

## Sequencing

Priority means current program urgency, while blocker edges mean a real prerequisite.
The graph does not make unrelated correctness and supply-chain fixes wait on each other
merely to force a serial work queue.

```text
Wave 0:  fdu-ad45 ──────────────────────────────────────┐
         fdu-nlh8 ──→ fdu-s7wr ──┐                   │
         fdu-1j0b ────────────────├─→ fdu-gd6n ────────├─→ fdu-sn43 ─→ PR #1 merged
         fdu-8jte ────────────────┘

Wave 1:  trust/API safety and corpus/oracle → runner → probes/adapters
Wave 2:  measured design decisions and risk spikes
Wave 3:  syscall walk, parallelism, packed reducers, and snapshot/revalidation
Wave 4:  product surfaces and stable evidence → final report → publishing
Future:  activate only from the evidence and release gates in the future roadmap
```

### Wave 0: Close the Current Merge Gate

Every implementation and final-validation node in this wave is closed.
The approval gate proved the assembled revision rather than any one fix:

- `fdu-ad45` restored the 14-day executable-dependency cool-off, provenance checks,
  least-privilege workflow settings, and reviewed tbd integration surfaces.
- `fdu-nlh8` validates a complete observation batch before mutation so malformed paths
  cannot look like no-ops or permit partial application.
- `fdu-1j0b` replaced writer-lock filesystem I/O with bounded apply-if-clock attempts;
  exhausted contention becomes an explicit root invalidation and normal reconciliation.
- `fdu-8jte` made the joined worker an I/O-free bounded coalescer.
  Verification occurs on the consuming thread; overload becomes a sticky root
  invalidation, and cancellation always joins the worker.
- `fdu-s7wr` followed atomic apply and removed lock guards, receivers, and lock-held
  callbacks from the supported API.
- `fdu-gd6n` followed all state/lifecycle fixes and proves whole-batch visibility,
  writer linearization, freshness epochs, snapshot replacement, watcher teardown, and
  Python thread behavior with deterministic interleavings.
- `fdu-l8vc`, `fdu-83gl`, and `fdu-ie5z` close the final thread-aware review findings:
  root-bound watch application, an explicit filesystem-sample convergence contract, and
  terminal-clock no-op/stale arbitration.
- `fdu-b3qe` keeps the online provenance gate authenticated locally and in CI without
  broadening pull-request permissions.
- `fdu-9xf7` corrects cfg-disabled integration-test documentation ordering; exact
  Windows-target compilation, the complete local gate, and fresh Windows CI pass.
- `fdu-sn43` completed the final local gate, fresh cross-platform CI, synchronized tbd
  state, PR description update, automated-thread disposition, and superseding senior
  approval after the supply-chain and concurrency validation gates.

No Phase 1 optimization is a substitute for closing this gate.

### Wave 1: Establish Safe Refactor and Evidence Foundations

The [Rust quality plan](plan-2026-08-09-fdu-rust-engineering-quality.md) has already
pinned and proved the normal/MSRV feature matrix (`fdu-zga3`). It next adds the
independent index model (`fdu-o8r8`) and snapshot fault-state suite (`fdu-471a`). The
guard-free API is already closed in Wave 0. Stack-safe rendering (`fdu-zsdy`) is
implemented in the focused CLI follow-up and remains blocked only on its validation
gate; lossless classification and Python identity (`fdu-k8zw`) follows the API work.

In parallel, the
[performance plan](plan-2026-08-09-fdu-end-to-end-performance-testing.md) builds one
shared evidence foundation instead of separate ad hoc benchmark scripts:

1. `fdu-rq5m`: deterministic corpora and semantic oracle;
2. `fdu-d8kq`: strict scenario/result schemas and state-machine runner;
3. `fdu-oj25`: fdu component probe and resource collectors;
4. `fdu-k5t5`: pinned dut/gdu adapters after the cool-off gate.

### Wave 2: Resolve Load-Bearing Decisions

The expensive implementation starts only after the evidence or design decision it
depends on:

- `fdu-p2i1` measures revalidation at 10k through 1M entries using the corpus, runner,
  and probe; it gates optimized revalidation.
- `fdu-1vd0` compares snapshot candidates using the same infrastructure; it gates the
  block format.
- `fdu-gdrv` proves whether metric-vector atomic roll-up remains barrier-free, including
  an explicit memory-ordering argument and model checking if the design stays lock-free;
  it gates parallel aggregation and the reducer registry.
- `fdu-p35d` measures tag-don’t-prune gitignore matching before the type-rule dialect.
- `fdu-odx6` records maintainer ratification or amendment of extensibility and
  trustworthy-result goals before their public interfaces are frozen.
- `fdu-579b` settles deterministic hardlink attribution before reducers and the snapshot
  encode that policy.

### Wave 3: Build the Optimized Engine

- `fdu-atqk` implements the portable-fallback syscall walker; `fdu-aky1` adds the
  measured, bounded, cancellable scheduler and parallel roll-up after the walker and
  refcount spike. It starts with safe scoped workers and a bounded queue; an intrusive
  unsafe queue needs separate measured and model-checked justification.
- `fdu-1gbl` packs records behind the API, model-test, and measurement foundations.
- `fdu-a6dz` implements registered reducers after the model, goal, refcount, and
  hardlink decisions.
- `fdu-xihx` implements the block snapshot only after its candidate spike, packed
  records, reducer encoding, and reusable persistence fault tests.
- `fdu-wbis` implements optimized revalidation after the cost curve and syscall walker.
- `fdu-r27g` measures the retained standard-library single-writer lock using the common
  probe before any synchronization redesign or dependency is considered.

### Wave 4: Finish Product Surfaces, Proof, and Release

- `fdu-oqoy` and `fdu-jej9` finish human and agent-facing CLI behavior after stack-safe
  rendering and native identity foundations.
- `fdu-v4lc` defines the native-unit type-rule dialect after gitignore measurement and
  goal ratification.
- `fdu-lka2` hardens platform backend failure, rename handling, descriptor limits, and
  reconciliation after the shared-index API and bounded generic transport are sealed.
- `fdu-8z5l` establishes stable regression and claim governance on the completed runner,
  probes, adapters, and pinned toolchain.
- `fdu-ywu0` publishes the full evidence matrix only after the required engine,
  contention, and harness work is complete.
- `fdu-9cf0` publishes crates and wheels only after the report and product surfaces pass
  their release gates; name availability is rechecked immediately before publication.

Content metrics, a durable delta journal, metabrowser integration, and io_uring are
owned by the
[post-Phase 1 roadmap](../future/plan-2026-08-09-fdu-post-phase-1-roadmap.md).

## Exit Criteria

Phase 1 is done when all of these hold, and not before:

1. Cold scan within ~1.5x of dut on the same corpus, with full stats retained.
2. Warm re-run (snapshot load + revalidation) well under 1 s for 500k entries.
3. Memory within ~25–32 bytes per file record.
4. `fdu --help` is complete enough that an agent needs no other documentation, and the
   JSON schema is versioned and stable.
5. The benchmark harness reports the full snapshot/filesystem-state ×
   producer/full-index matrix, and the README’s performance claims cite its raw evidence
   and reproduction manifest.
6. Goals 6 and 7 are ratified or amended — they already shape the architecture, so
   leaving them unsigned means building on an unratified premise.
7. Every concurrent subsystem has bounded ownership, deterministic shutdown/error tests,
   and no filesystem I/O, blocking send, or user callback under an index lock; custom
   lock-free protocols have a documented memory model and model-checking evidence.

## Open Questions

Carried from the research, still unanswered, each one a decision someone has to make:

1. **Hardlink attribution (`fdu-579b`)** under incremental updates — no prior art to
   copy.
2. **io_uring (`fdu-ktka`)**: phase-1 complexity or a later accelerator behind a feature
   flag? Large machinery for a Linux-only win.
3. **DFS/BFS traversal order (`fdu-aky1`, `fdu-ywu0`)** — worth a runtime switch, and
   can warm/cold state be detected rather than configured?
4. **Content probe bounds (`fdu-v4lc`, `fdu-3n8c`)** for type recognition (first 8
   KiB?), and whether sniffing is on-demand-only at first so the walk stays stat-pure.
5. **Engine versus Python classification (`fdu-v4lc`, `fdu-p02b`)**. Proposal: the
   engine yields type verdicts from compiled rules; adapter-level sniffing stays in
   plugins. Validate against real manifests.
6. **Journal compaction and `since(clock)` across restart (`fdu-3dtq`)** — nice for SSE
   resume, simpler without.
7. **Non-invertible reducer cost under churn (`fdu-a6dz`, `fdu-oj25`)** — measure the
   pathological case before deciding which metrics are watch-maintained versus
   revalidation-only.
8. **Watcher ownership in metabrowser (`fdu-p02b`, `fdu-lka2`)**: does `fdu::watch`
   replace `watch_backends.py` outright, or does metabrowser retain its watcher and push
   hints through `ingest_events()`? The open part is sequencing and the acceptance test
   for dropping the Python watcher.

## Note for Metabrowser

One research finding applies to metabrowser today, independent of whether fdu ever
ships: `watchfiles` maps notify’s event model down to `(change, path)` and drops the
`Rescan` flag on the way, so after a burst large enough to overflow kernel queues — a
`git checkout`, an `npm install` — the Python inventory can silently diverge until
restart. That is tracked separately in the metabrowser repository.

## Beads

Epic: **fdu-qfz6** — fdu phase 1: fastest walker with full stats, proven by benchmark.
The status-reconciliation record is **fdu-co2i**. Phase 0 and its final review follow-up
are recorded as closed beads **fdu-v178** and **fdu-vdi9**; the completed CLI workstream
is **fdu-a0w0**.

### Current Merge Gate

| Bead | Priority | State | Work | Direct blockers |
| --- | --- | --- | --- | --- |
| `fdu-ad45` | P0 | Closed | Restore and enforce executable-dependency cool-off and provenance | — |
| `fdu-nlh8` | P0 | Closed | Reject malformed observation batches atomically | — |
| `fdu-1j0b` | P1 | Closed | Remove filesystem I/O from watch writer-lock arbitration | — |
| `fdu-8jte` | P1 | Closed | Bound watcher overload, cancellation, and shutdown | — |
| `fdu-s7wr` | P1 | Closed | Seal the guard-free ownership API | `fdu-nlh8` |
| `fdu-gd6n` | P1 | Closed | Prove concurrency contracts deterministically | `fdu-s7wr`, `fdu-1j0b`, `fdu-8jte` |
| `fdu-l8vc` | P0 | Closed | Bind supported watch application to the indexed root | — |
| `fdu-83gl` | P0 | Closed | Specify watch stat-to-commit linearization and convergence | — |
| `fdu-ie5z` | P0 | Closed | Preserve no-op and stale terminal-clock arbitration | — |
| `fdu-b3qe` | P0 | Closed | Authenticate live provenance checks with least privilege | — |
| `fdu-9xf7` | P0 | Closed | Keep cfg-disabled integration-test crates documented cross-platform | — |
| `fdu-sn43` | P0 | Closed | Run final gates and publish the superseding senior approval | `fdu-ad45`, `fdu-gd6n`, `fdu-l8vc`, `fdu-83gl`, `fdu-ie5z`, `fdu-b3qe`, `fdu-9xf7` |

The implementation beads are children of the Rust-quality epic.
Independent fixes are not serialized; `fdu-gd6n` is the convergence point.
`fdu-sn43` is closed; its completion is the explicit approval record and start gate for
the queued Phase 1 work.

### Governing Workstreams

| Epic | Status | Start blocker | Governing plan |
| --- | --- | --- | --- |
| `fdu-qfz6` | Active | — | This Phase 1 plan |
| `fdu-dxee` | Active; owns Wave 0 | — | [Rust engineering quality](plan-2026-08-09-fdu-rust-engineering-quality.md) |
| `fdu-6c8n` | Active follow-up | `fdu-sn43` | [CLI UX and zero-install skill](plan-2026-08-09-fdu-cli-ux-and-agent-skill.md) |
| `fdu-d5e1` | Queued | `fdu-sn43` | [End-to-end performance evidence](plan-2026-08-09-fdu-end-to-end-performance-testing.md) |
| `fdu-x746` | Future | `fdu-9cf0` | [Post-Phase 1 roadmap](../future/plan-2026-08-09-fdu-post-phase-1-roadmap.md) |

The two detailed active plans list every child bead they own and every cross-workstream
blocker. The table below is the complete set owned directly by this plan.

### Phase 1 Execution Beads

| Wave | Bead | Priority | Work | Direct blockers |
| --- | --- | --- | --- | --- |
| 0 | `fdu-sn43` | P0 | Close the PR #1 merge gate (complete) | `fdu-ad45`, `fdu-gd6n`, `fdu-l8vc`, `fdu-83gl`, `fdu-ie5z`, `fdu-b3qe`, `fdu-9xf7` |
| 2 | `fdu-gdrv` | P1 | Prove metric-vector atomic-refcount roll-up | `fdu-sn43` |
| 2 | `fdu-p35d` | P1 | Measure gitignore tag-don’t-prune matching | `fdu-sn43` |
| 2 | `fdu-odx6` | P1 | Ratify or amend goals 6 and 7 | `fdu-sn43` |
| 2 | `fdu-579b` | P1 | Set deterministic incremental hardlink attribution | `fdu-sn43` |
| 2 | `fdu-p2i1` | P1 | Measure the 10k-1M revalidation cost curve | `fdu-rq5m`, `fdu-d8kq`, `fdu-oj25` |
| 2 | `fdu-1vd0` | P1 | Compare snapshot open and first-listing candidates | `fdu-rq5m`, `fdu-d8kq`, `fdu-oj25` |
| 3 | `fdu-atqk` | P1 | Implement `getdents64` and dirfd-relative `statx` | `fdu-oj25` |
| 3 | `fdu-aky1` | P1 | Add work-stealing parallel walk and roll-up | `fdu-gdrv`, `fdu-atqk` |
| 3 | `fdu-1gbl` | P1 | Pack entry records to the measured memory budget | `fdu-s7wr`, `fdu-o8r8`, `fdu-oj25` |
| 3 | `fdu-a6dz` | P1 | Implement the reducer registry and overflow policy | `fdu-gdrv`, `fdu-s7wr`, `fdu-o8r8`, `fdu-odx6`, `fdu-579b` |
| 3 | `fdu-xihx` | P1 | Implement the block snapshot format | `fdu-1vd0`, `fdu-1gbl`, `fdu-a6dz`, `fdu-471a`, `fdu-579b` |
| 3 | `fdu-wbis` | P1 | Optimize revalidation and stream applied deltas | `fdu-p2i1`, `fdu-atqk` |
| 3 | `fdu-r27g` | P2 | Measure index contention before changing synchronization | `fdu-s7wr`, `fdu-oj25` |
| 4 | `fdu-oqoy` | P2 | Finish human CLI behavior | `fdu-zsdy`, `fdu-6c8n` |
| 4 | `fdu-jej9` | P2 | Finish agent CLI and schema behavior | `fdu-zsdy`, `fdu-k8zw`, `fdu-6c8n` |
| 4 | `fdu-v4lc` | P2 | Define native-unit compiled type rules | `fdu-k8zw`, `fdu-p35d`, `fdu-odx6` |
| 4 | `fdu-lka2` | P2 | Harden watcher platform backends | `fdu-s7wr`, `fdu-8jte` |
| 4 | `fdu-9cf0` | P2 | Publish crates and wheels after all release gates | `fdu-ad45`, `fdu-zga3`, `fdu-s7wr`, `fdu-k8zw`, `fdu-ywu0`, `fdu-6c8n`, `fdu-oqoy`, `fdu-jej9`, `fdu-v4lc`, `fdu-lka2` |

The future epic owns `fdu-p02b`, `fdu-3dtq`, `fdu-3n8c`, and `fdu-ktka`; their explicit
activation dependencies are recorded in the future roadmap rather than mixed into the
active queue.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
