# Feature: FSEvents-Scoped Revalidation (macOS)

**Date:** 2026-08-10

**Author:** fdu project

**Status:** Draft

## Overview

Make a warm start on macOS cost O(changes) instead of O(tree) by replaying a
device-relative FSEvents stream from the last boundary whose work was applied, then
revalidating only normalized filesystem scopes.

Today a warm start is a strict superset of a cold scan: load the snapshot, build the
index, then walk the entire tree again to prove the snapshot still describes it.
The full sweep is the only sound revalidation the filesystem itself offers, because
change information does not propagate upward through directory mtimes — an in-place file
edit is invisible to every ancestor directory, including its immediate parent (verified
empirically on APFS; see the
[change-propagation analysis](../../research/research-2026-08-10-performance-frontier.md)).
Beating the per-entry stat floor therefore requires an operation log, and macOS has one
on disk already: `fseventsd` journals change events persistently, across
process exits and reboots.
Storing an applied journal boundary in the snapshot can avoid most of that sweep. This
is a hypothesis until the corrected state machine is implemented and measured; no
size-independent latency claim follows from the current experiments.

This is rung 2 of the warm ladder in the
[performance-frontier research](../../research/research-2026-08-10-performance-frontier.md)
(backlog item H43). One honesty note that research’s source-level read established and
this plan inherits: Watchman’s `fsevents_try_resync` proves the *mechanics* (resume from
a recorded event ID, UUID-guarded, wrap-vetoed) but uses them only for in-process
recovery, off by default — and across restarts both Watchman and git’s fsmonitor daemon
start at `SinceNow` and re-crawl.
Cross-restart replay is Apple-documented and API-supported but unproven in major
production tools; fdu would be pioneering it.
That is why the validation spike is Phase 0 rather than an afterthought, why the gate
fails closed on every row, and why the full sweep remains the backstop on every platform
— the only rung correctness ever depends on.

## Goals

- Persist an FSEvents boundary (platform tag, volume UUID, applied event ID, capture
  time, last full-sweep time) with the scan or replay transaction that established it
- On load, when a strict gate passes, replay the journal since the cursor and revalidate
  only the named directories, emitting ordinary conditional deltas
- Use `FSEventStreamCreateRelativeToDevice` with `FullHistory`, `FileEvents`, and
  `WatchRoot`; normalize item events into directory relists or subtree invalidations
- Map every journal degradation signal onto the existing escalation vocabulary: scoped
  `InvalidateSubtree` where the flag is scoped, full sweep where it is not
- Persist only the maximum replay boundary whose scopes were applied; never sample a
  newer current ID during save
- Keep journal-scoped freshness distinct from full-sweep freshness, and require
  periodic full sweeps because Apple documents the event list as advisory
- Compare both paths with the same entry and roll-up oracle in every trial
- Land the numbers through the performance loop as experiments, with the accept rule
  deciding

## Non-Goals

- Windows (USN journal) and Linux: Windows is deferred with the format leaving room for
  it; Linux has no persistent journal, which is exactly why the parallel sweep must stay
  fast there. The two investments are complements, not alternatives.
- Changing what is cached or where.
  The snapshot is written on every platform today and stays that way: platform-neutral
  caching is what makes content-tier roll-ups (line counts, hashes — reducers that must
  re-read changed files) worth building later, on every OS. This feature only
  accelerates *revalidation*, on one OS.
- Touching the live watch layer.
  The watcher (rung 3) already exists behind the `watch` feature; this is the
  between-runs story, not the resident one.
- Spotlight or any other time-indexed query source.
  Deletions have no mtime, so “changed since T” is answerable only from an operation
  log.
- The block snapshot format (H33/H35, bead `fdu-1vd0`). The cursor fields ride the
  current flat format now and carry over unchanged when that lands.
- Trusting the journal.
  Apple documents FSEvents as advisory.
  Every use here is a *scoping hint* that decides where to look; what the index believes
  still comes only from fresh stats through the delta contract.

## Background

Measured on a 59,654-entry real checkout (Apple M1 Pro, APFS, warm page cache):

| path | wall | of which |
| --- | ---: | --- |
| cold scan into index | ~320 ms | walk 170 ms + index build ~90 ms |
| warm snapshot load | ~230 ms | parse + index build |
| warm revalidate (load + full sweep) | ~690 ms | the sweep re-pays the whole walk |

The warm path is a strict superset of the cold path by construction, which makes the
snapshot cache a net loss for a one-shot query today.
Three experiments (exp-002, exp-004, exp-005 in the
[ledger](../../reports/report-2026-08-10-fdu-performance-experiments.md)) improved
constants; none can change the asymptotics, because the sweep must stat every entry to
be sound. Only change information can — and `fseventsd` records it in persistent
per-volume journals. For persistent software Apple recommends a per-disk stream created
with `FSEventStreamCreateRelativeToDevice`; IDs share a system-wide sequence, but the
stream, UUID, and retained history are volume-bound. Replay uses a stored ID as
`sinceWhen`, with explicit flags for every way the history can be insufficient
(`kFSEventStreamEventFlagMustScanSubDirs`, `UserDropped`, `KernelDropped`,
`EventIdsWrapped`, `HistoryDone`, `RootChanged`, `Mount`, `Unmount`).

This design deliberately requests `FileEvents`, so callbacks name filesystem items,
not changed directories. File and symlink events normalize to a parent-directory
relist. Directory create/remove/rename events normalize to a parent relist plus a
subtree invalidation when the directory still exists. Ambiguous item flags fall back
to the full sweep. That normalization—not an assumption about callback granularity—is
what supplies revalidation scopes.

### What the journal is worth, honestly

Against today’s serial sweep the journal looks overwhelming: ~690 ms → tens of
milliseconds on a quiet 60k tree.
That comparison flatters it, and the
[frontier research’s calibration](../../research/research-2026-08-10-performance-frontier.md)
is the honest one. Rung 1 of the warm ladder — producer-side no-op elision (the
registry’s H12), the parallel sweep, and eventually bulk stat (H26) — is expected to
bring a warm-cache revalidation down toward parallel-producer time on its own: ~190 ms
at 60k today, and ~0.2–0.4 s per million entries on a warm cache.
Measured against *that* baseline, the journal at 60k-warm is an incremental win, not a
transformative one.

The largest plausible wins are cold metadata caches, remote storage, and very large
local trees, because each avoided stat may avoid a real I/O or round trip. Those cells
have not been measured. They remain explicit matrix hypotheses, including the
whole-drive scenario (H45), rather than supporting claims today.

Both documents draw the same conclusion from opposite ends: the sweep must stay fast
regardless, because the journal degrades into it (gate rows G3–G9), and the journal must
exist, because no sweep reaches O(changes).
The loop’s accept rule will judge Phase 2 against the rung-1 baseline current at
measurement time, not against today’s serial sweep.

Scoped revalidation also composes with the platform work rather than competing with it:
once bulk stat lands (H26), re-verifying a named changed directory is one
`getattrlistbulk` call — the research’s whole-drive composition is journal resume (H43)
naming the directories, bulk re-scan (H26) verifying them, and persisted roll-ups
(H33/H16) rendering the rest untouched.

### How modern Rust talks to FSEvents (researched 2026-08-10)

The scheduling API matters more than the binding crate.
`FSEventStreamScheduleWithRunLoop` — what `notify` 8.2 and every older example uses — is
**deprecated since macOS 13**; Apple’s replacement is `FSEventStreamSetDispatchQueue`
(available since 10.6), which delivers callbacks on a GCD queue and needs no run loop,
no dedicated thread parked in `CFRunLoopRun`, and no cross-thread `CFRunLoopStop`
teardown dance. For a one-shot historical replay the dispatch-queue form is also simply
the better shape: create stream → set queue → start → wait on a channel for
`HistoryDone` or deadline → stop/invalidate/release.

Binding options, evaluated against this repository’s supply-chain policy:

| Option | New crates in the locked tree | Notes |
| --- | --- | --- |
| `fsevent-sys 4.1.0` (already present via `notify`) + own declarations | **0** | Ships `FSEventStreamCreate`, start/stop/invalidate/release, `FSEventsGetCurrentEventId`, and every event flag. Leaves `FSEventStreamSetDispatchQueue` and `FSEventsCopyUUIDForDevice` commented out; both are declared in our FFI module, alongside `dispatch_queue_create`/`dispatch_release` (libdispatch is part of libSystem, linked on every macOS binary — no crate needed). |
| `objc2-core-services` + `objc2-core-foundation` + `dispatch2` | ~4–5 | The modern generated bindings (verified: all four needed functions exist, including `FSEventStreamSetDispatchQueue` and `FSEventsCopyUUIDForDevice`). Signatures machine-derived from Apple headers. |

**Decision: option 1.** The deciding fact is that `fsevent-sys` is *already in
`Cargo.lock`*, so the entire feature adds zero new crates and nothing to the cool-off,
while still using the non-deprecated dispatch-queue API — the handful of extern
declarations we add are exactly the ones the objc2 crates would generate, and they are
covered by the same integration tests either way.
The objc2 route is the documented fallback if the hand-declared surface grows past a
dozen functions; it is a maintained, widely-used ecosystem (winit), just not worth four
new supply-chain entries for ~6 declarations today.

The workspace denies `unsafe_code`; the one FFI module carries a scoped
`#[allow(unsafe_code)]` with every call site documented, and no unsafe appears anywhere
else. This is a far smaller decision than the still-blocked `libc`-for-`openat` question
(H2/H24): same locked crate set, unsafe confined to one leaf module behind a non-default
feature on one platform.

Replay semantics that the implementation and its tests must honor, from Apple’s
documentation and Watchman’s source (mechanics only — see the Overview’s honesty note on
how far its production use actually goes):

- The required stream flags are `FullHistory | FileEvents | WatchRoot`.
  `FullHistory` deliberately permits the first historical chunk to overlap the stored
  boundary, including events whose IDs are less than or equal to `sinceWhen`, so replay
  must be idempotent and must not assume strict-greater delivery.
- The end of history is a sentinel event flagged `HistoryDone` whose path is meaningless
  and must be ignored. Events after that sentinel belong to the live stream and are not
  part of this replay transaction.
- Event IDs are allocated from a machine-wide monotonic counter, but the journals they
  index — and their retention — are per-volume.
  A boundary is therefore useful only together with the volume UUID (from
  `FSEventsCopyUUIDForDevice(st_dev)`, never `st_dev` itself, which is not stable across
  reboots). `FSEventsGetCurrentEventId()` supplies an initial system-wide fence; it is
  not a per-volume retention test. UUID loss or change and `EventIdsWrapped` fail closed.
- Stream `latency` may be 0 for replay; coalescing within the historical log has already
  happened.
- Callbacks arrive on the dispatch queue with C types; the callback marshals into a
  plain `Vec<(PathBuf, flags, event_id)>` behind a channel and does nothing else — all
  logic stays in safe code on the calling thread.

## Design

### Where it fits

The architecture invariant survives untouched: **the journal is a producer of scope,
never of state.** Nothing new mutates the index.

```
snapshot.load ──► index          (as today)
      │
      ▼
journal gate ──► pass ──► replay applied boundary ──► changed-dir set
      │                                              │
      └─► fail ──► full sweep (as today)             ▼
                                     scoped revalidate: re-list + stat
                                     ONLY the named directories, emit
                                     conditional Upsert/Remove deltas
                                              │
                                              ▼
                                     index.apply  (unchanged contract)
                                              │
                                              ▼
                                     snapshot.save with applied boundary
```

The scoped revalidation reuses the sweep’s own emission logic bounded to a directory
set: for each named directory, re-list it, stat its immediate children, emit conditional
upserts and removals against the index’s recorded expectations — the same ops the full
sweep would have emitted for those directories, with the same ABA arbitration on apply.
A `MustScanSubDirs` flag on a path becomes `InvalidateSubtree(path)` resolved by the
existing subtree reconcile; the escalation vocabulary already fits because the watch
layer needed it first.

The initial snapshot and warm replay are explicit transactions:

1. Canonicalize the root, obtain its device and volume UUID, and reject a null UUID or
   a scope that crossed devices.
2. Immediately before the population scan, capture `B0 =
   FSEventsGetCurrentEventId()` and bind it to that UUID. The macOS implementation also
   creates and starts the device-relative stream before walking, following Apple’s
   scan-then-replay guidance; callbacks may queue while the scan runs.
3. Complete the full scan and persist its result together with `B0`. Never sample a
   replacement boundary during snapshot serialization. Any mutation racing the scan is
   at or after the fence and is reverified on the next open.
4. On a warm open, create a device-relative stream for the root’s volume and root path,
   starting at the stored boundary with `FullHistory | FileEvents | WatchRoot`.
5. Collect through `HistoryDone`. Harmless overlap at or before the stored boundary is
   normalized and revalidated along with newer events. Initialize the candidate applied
   boundary to the stored value and advance it only to the maximum non-sentinel event ID
   included before `HistoryDone`.
6. Apply every normalized scope through ordinary deltas. Only after all applications
   succeed may the snapshot persist that candidate boundary. A crash or error preserves
   the older boundary; an event arriving after `HistoryDone`, during revalidation or
   save, remains beyond the applied boundary and is replayed on the following open.

Item normalization is conservative. A file, symlink, or hard-link event relists its
parent. A directory create, remove, or rename relists its parent and invalidates the
existing or newly visible subtree. A scoped `MustScanSubDirs` without a dropped-event
flag invalidates that subtree. An undecodable or out-of-root path, contradictory item
flags, root change, mount transition, journal drop, wrap, or other unscoped ambiguity
forces a full sweep.

### The gate

The user-visible rule is “journal when provably applicable, sweep otherwise,” and the
gate is a pure decision function so the whole table is unit-testable without
CoreServices. Every row falls closed to the sweep:

| # | Condition | Decision |
| --- | --- | --- |
| G1 | Not macOS, feature off, or `--revalidate=full` | full sweep |
| G2 | Snapshot has no applied boundary (older format, or first save) | full sweep; capture the next boundary before scanning |
| G3 | Volume UUID is null or differs, or the root scope crosses devices | full sweep |
| G4 | `FullHistory` or a device-relative stream is unavailable on this macOS version | full sweep |
| G5 | Last verified full sweep is older than `max_full_sweep_interval` (default 24 hours) | full sweep — Apple documents events as advisory |
| G6 | Stream creation fails or replay exceeds `replay_timeout` (default 5 s) without `HistoryDone` | full sweep |
| G7 | Replay reports `EventIdsWrapped`, `RootChanged`, `Mount`, `Unmount`, `UserDropped`, `KernelDropped`, unavailable history, an undecodable/out-of-root path, or ambiguous flags | full sweep |
| G8 | Replay reports `MustScanSubDirs(path)` without an unscoped G7 signal | scoped: `InvalidateSubtree(path)`, journal continues for the rest |
| G9 | Changed-dir set exceeds `max_changed_fraction` (default 25%) of the snapshot’s directories | full sweep — scoped work would approach sweep cost with worse locality |
| G10 | Otherwise | scoped revalidation; report journal-scoped rather than full-sweep freshness |

The changed-dir set is normalized before G9: paths outside the root are dropped,
descendants of a `MustScanSubDirs` subtree are absorbed into it, duplicates coalesce,
and paths are mapped root-relative against the same canonicalized root the scan layer
uses.

Volume identity is the UUID, not `st_dev` — device numbers are not stable across
reboots. Event IDs come from a system-wide sequence, while stream history and UUID are
volume-bound, and a snapshot that spans a mount boundary cannot use a single boundary.
Phase 1 sidesteps this by combining the gate with the existing `one_filesystem` scope
information: a snapshot whose scan crossed devices simply never carries a boundary
(G2/G3). There is intentionally no “stored ID versus current volume ID” gate: the API
does not provide such a per-volume value, and the system-wide current ID says nothing
about whether one volume retained the requested history.

Because Apple calls the event list advisory, journal-scoped replay does not earn the
same freshness label as a complete stat sweep. The snapshot records
`last_full_sweep_at`; the default automatic policy performs a full sweep at least every
24 hours. Until Phase 0 validates a product-appropriate advisory mode, an interface that
promises exact current state must continue to select the full sweep. A later opt-in
journal mode may expose
`Freshness::JournalScoped { applied_through, last_full_sweep_at }` explicitly rather
than silently calling it fresh.

### Snapshot format

`FORMAT_VERSION` 2 → 3. After the scope header, one new optional section:

```
journal_cursor: u8 tag        0 = none, 1 = fsevents-v1  (room for usn-v1 = 2)
if fsevents-v1:
  volume_uuid: 16 bytes
  applied_event_id: u64       initial pre-scan fence or max replay ID actually applied
  captured_at: i64 ns         wall clock when that boundary transaction began
  last_full_sweep_at: i64 ns  wall clock of the most recent complete stat sweep
```

The initial boundary is captured **before** the scan that populates the index begins,
not after it ends. Replay advances it only after every normalized scope through that ID
has been applied. Snapshot serialization receives this metadata from the completed scan
or replay session; it never calls FSEvents itself.
A v2 snapshot loads as cursor-absent (G2), not as invalid — the usual
corrupt-fails-closed rules are unchanged, and a corrupt cursor section discards the
snapshot like any other parse failure.

### Components

- `crates/fdu/src/journal/mod.rs` — platform-neutral surface: `JournalCursor`
  (encode/decode), `GateDecision`, the gate function, changed-set normalization.
  Compiles everywhere; no FFI.
- `crates/fdu/src/journal/fsevents.rs` — `#[cfg(target_os = "macos")]`, feature
  `journal`. The FFI module: initial system-wide fence, volume UUID for a device, and
  historical replay via the non-deprecated dispatch-queue API (create a
  device-relative stream with the required flags and `sinceWhen`,
  `FSEventStreamSetDispatchQueue` onto a private queue, start, receive marshalled
  `(path, flags, event_id)` tuples over a channel until `HistoryDone` or the G6 deadline, then
  stop/invalidate/release).
  The only module in the workspace allowed `unsafe`.
- `crates/fdu/src/scan.rs` — `revalidate_dirs(index, dirs, config, sink)`: the bounded
  sweep. Reuses the existing per-directory emission; no new op kinds.
- `crates/fdu/src/snapshot.rs` — format v3 fields supplied by the completed scan or
  replay session; no platform cursor capture inside serialization.
- `crates/fdu/src/cli.rs` — wire the gate into the cached path; `--revalidate=auto|full`
  (default `auto`; `full` forces the sweep).
  `--no-cache` is untouched.
- Feature `journal` in `crates/fdu/Cargo.toml`: gates `dep:fsevent-sys` (macOS only via
  target-conditional dependency) and the FFI module.
  Off by default initially; the CLI enables it once the evidence is in.
  On non-macOS targets the feature compiles to the gate returning G1, so
  `--no-default-features` and Linux/Windows builds are unaffected.

### API changes

Additive, but not signature-free. Saving cannot capture a correct boundary internally:
the value must be bound to the scan or replay whose effects are present in the index.
Add a `SnapshotMetadata` (or equivalently named scan-session result) carrying the
optional `AppliedJournalBoundary`, `last_full_sweep_at`, and freshness. A new
`snapshot::save_with_metadata` accepts it; the existing `snapshot::save` remains a
compatibility wrapper that writes no journal boundary. Loading returns the metadata
alongside the index through a new API while the compatibility load path may discard it.
The other new public surface is `JournalCursor`, `GateDecision`, and
`scan::revalidate_dirs`, documented as accelerator plumbing with the sweep as the
portable contract.

### Packaging and platform fallback

One source tree, one feature name, correct behavior on every platform without the
consumer doing anything:

- The `journal` feature exists on **all** platforms.
  On macOS it compiles the FFI module and the gate can return scoped decisions;
  elsewhere it compiles only the platform-neutral gate, whose first row (G1) answers
  “full sweep.” Enabling the feature is therefore never a build error and never changes
  non-macOS behavior — the fallback is the same code path Linux runs today, not a stub.
- The dependency is target-conditional:
  `[target.'cfg(target_os = "macos")'.dependencies] fsevent-sys = { version = "4.1", optional = true }`.
  Linux and Windows builds with `--features journal` pull no new crates at all.
- **Cargo consumers**: `default-features = false` builds are unaffected; the CLI build
  turns the feature on once the evidence gate passes.
- **PyPI / uv consumers**: `fdu-py` wheels are built per-platform by maturin, so the
  macOS wheels carry the journal path and the manylinux wheels carry the fallback, from
  the same source with no Python-side conditionals, extras, or environment markers.
  `uv pip install fdu` (or `uvx fdu`) gets the right behavior on either OS because the
  platform selection already happened at wheel-build time — the same mechanism that
  ships every other platform difference today.
- Both distribution channels are exercised in CI: the existing test matrix
  (ubuntu/macos/windows) proves the feature compiles and falls back everywhere, and the
  wheel legs install the built wheel and run the warm-path smoke on each OS.

## Implementation Plan

Sequencing follows the research’s optimization ladder rather than jumping it.
That ladder places journal resume in its final rung — every accelerator needs the lower
rungs as its fallback — while its measurement principle is “measure early, implement in
ladder order,” and it explicitly recommends reserving the snapshot fields now.
So the cheap, information-producing phases run immediately (the spike measures, the
format phase reserves), and the full implementation lands when the loop’s rung-1 warm
work (H12/H14) has produced the clean baseline Phase 2 must be judged against.

### Phase 0: Validation spike (macOS, throwaway)

Every load-bearing implementation assumption is validated by a disposable probe before
real code depends on it.
The spike binary lives outside the shipped surfaces (an uncommitted scratch crate or a
gitignored example), links `fsevent-sys` from the existing lockfile plus the
self-declared externs, and answers, on a real volume:

- [ ] Dispatch-queue delivery works as designed: stream created with `sinceWhen`,
  `FSEventStreamSetDispatchQueue` onto a private queue, events arrive, the `HistoryDone`
  sentinel arrives, teardown is clean, and a deadline abort works (gate G6’s mechanism)
- [ ] Replay semantics hold: `FullHistory` overlap at or before `sinceWhen` is harmless;
  `FileEvents` names individual items; file edits, directory create/remove/rename,
  symlinks, and hard links all normalize to the expected parent relists or subtree
  invalidations; replay of an hour-old and a week-old boundary completes quickly
- [ ] Degradation is observable: a `sinceWhen` predating journal retention produces
  `MustScanSubDirs` (or equivalent) rather than silent emptiness — the G7/G8 inputs are
  real
- [ ] Volume identity works: `FSEventsCopyUUIDForDevice` for the root’s current `st_dev`
  returns a stable UUID across remount; the self-declared extern links
- [ ] Permission surface is understood: what a plain user process sees for its own trees
  without Full Disk Access, and whether any TCC prompt appears
- [ ] Cross-restart reliability is probed directly, because this is the unproven part:
  cursor written by one process, replay by a fresh process; replay after logout or
  reboot where practical; mutations made while no fdu process exists are the ones the
  replay must name. Any observed loss that arrives *without* a degradation flag is a
  finding that changes the design (a mandatory periodic full sweep gets promoted from
  paranoia to contract), and is exactly what the Watchman revert suggests looking for
- [ ] Events racing the initial scan, historical replay, scoped revalidation, and
  snapshot save are each observed on the next applicable transaction; no test advances
  the stored boundary beyond applied work
- [ ] Findings are recorded by amending this spec’s gates and constants (G5/G6 defaults,
  G9 fraction) and noted in the experiment ledger; explicit go/no-go for Phase 2

### Phase 1: Format and gate (mergeable alone; unblocks the block-format spike)

- [ ] Snapshot format v3: boundary section and full-sweep timestamp, metadata-aware save
  API (writes `none` without session metadata), load-side decode, corrupt section fails
  closed
- [ ] `journal/mod.rs`: cursor types, gate decision table as a pure function,
  changed-set normalization; exhaustive unit tests for every gate row
- [ ] Golden and round-trip tests: v2 loads as cursor-absent; v3 round-trips

### Phase 2: Replay and scoped revalidation (macOS)

- [ ] `journal/fsevents.rs`: FFI declarations, current-event-id, volume UUID, historical
  replay with deadline; scoped `#[allow(unsafe_code)]` with per-call safety comments
- [ ] `revalidate_dirs` in scan.rs, plus `InvalidateSubtree` resolution for G8
- [ ] CLI: gate wiring, `--revalidate` flag, initial pre-scan fence, and applied-boundary
  propagation on macOS
- [ ] Integration tests (macOS CI leg): mutate-then-journal-revalidate equals fresh scan
  by the full per-directory roll-up digest; UUID mismatch/null, crossed devices,
  unavailable history, `EventIdsWrapped`, dropped events, root/mount changes, permission
  failure, replay deadline, and forced `MustScanSubDirs` each degrade correctly
- [ ] Race tests cover initial scan, replay collection, scoped apply, and snapshot save;
  a crash before apply completion preserves the older boundary, and a mutation after
  `HistoryDone` is replayed on the next open
- [ ] Periodic-sweep tests prove the 24-hour limit and the distinction between
  full-sweep and journal-scoped freshness
- [ ] Cross-platform packaging: target-conditional dependency, `journal` feature
  compiling on every platform with the G1 fallback, ubuntu CI leg running the fallback
  end-to-end with digest equality, wheel smoke exercising a warm open through Python on
  both OSes
- [ ] Performance loop: new `warm-revalidate-journal` job; the one-deep-edit acceptance
  scenario on the reference tree (quiet, one-file-touched at depth ≥ 10, and
  one-file-deleted rows, full sweep as the paired control) and a churn transition
  (expect cost ∝ changes, not size — H43/H38); ledger entries either way; feature stays
  off by default until the loop accepts

## Testing Strategy

The gate is a pure function: every row in the table gets a direct unit test, no platform
APIs involved.
Replay is integration-tested only on macOS (`#[cfg]`-gated, running in the
existing macos-latest CI leg): each test mutates a real temp tree between snapshot and
reopen, then asserts the journal-scoped index equals a fresh scan’s by engine digest —
the same full per-directory roll-up equality the benchmark oracle and parallel-walker
tests pin. Degradations are forced: null/wrong UUID, crossed volume, unavailable history,
every dropped/wrapped/root/mount flag, bad path decoding, timeout, stale full-sweep
timestamp, and an undersized `max_changed_fraction`. Race tests place mutations on each
side of every boundary transition. The performance harness needs no oracle changes: a
journal-path trial that skips a real change fails the run loudly.
Linux and Windows CI prove the feature compiles away cleanly.

## Rollout Plan

Feature `journal`, off by default, on for the CLI build once both phases pass the gate
*and* the loop’s experiments accept.
The README and skill text may only claim what the ledger shows, per the existing
no-unmeasured-claims convention.

## Open Questions

- How much does `FileEvents` expand under high-frequency mixed workloads after
  conservative parent/subtree normalization? The churn experiment answers it
  empirically; G9 bounds the damage if the answer is unfavorable.
- Cursor-per-volume for multi-volume scans: deferred behind G2 + `one_filesystem` now;
  the tag byte leaves room for a multi-cursor section later.
- Should `--revalidate=journal` exist (fail rather than sweep when the gate refuses)?
  Useful for testing; possibly confusing as a user surface.
  Deferred until the integration tests want it.

## References

- [Performance-frontier research](../../research/research-2026-08-10-performance-frontier.md)
  — change-propagation physics, warm ladder, H43, macOS findings
- [Performance loop guide](../../guides/performance-loop.md) and
  [experiment ledger](../../reports/report-2026-08-10-fdu-performance-experiments.md)
- [FSEvents Programming Guide (persistent event IDs)](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html)
- [FSEventStreamCreateFlagFullHistory](https://developer.apple.com/documentation/coreservices/kfseventstreamcreateflagfullhistory)
- [FSEventStreamCreateRelativeToDevice](https://developer.apple.com/documentation/coreservices/1443980-fseventstreamcreaterelativetodevice)
- [FSEventStreamSetDispatchQueue](https://developer.apple.com/documentation/coreservices/1444164-fseventstreamsetdispatchqueue)
  — the non-deprecated scheduling API;
  [`ScheduleWithRunLoop` deprecation reports](https://github.com/fsnotify/fsevents/issues/59)
- [objc2-core-services](https://crates.io/crates/objc2-core-services) — the generated
  modern bindings, evaluated and documented as the fallback route
- [Watchman fsevents resync](https://facebook.github.io/watchman/docs/troubleshooting.html)
- [End-to-end performance testing plan](plan-2026-08-09-fdu-end-to-end-performance-testing.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
