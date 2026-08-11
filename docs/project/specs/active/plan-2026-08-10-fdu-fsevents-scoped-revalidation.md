# Feature: FSEvents-Scoped Revalidation (macOS)

**Date:** 2026-08-10

**Author:** fdu project

**Status:** Draft

## Overview

Make a warm start on macOS cost O(changes) instead of O(tree) by replaying the FSEvents
persistent journal since the snapshot was written, and revalidating only the directories
it names.

Today a warm start is a strict superset of a cold scan: load the snapshot, build the
index, then walk the entire tree again to prove the snapshot still describes it.
The full sweep is the only sound revalidation the filesystem itself offers, because
change information does not propagate upward through directory mtimes — an in-place file
edit is invisible to every ancestor directory, including its immediate parent (verified
empirically on APFS; see the
[change-propagation analysis](../../research/research-2026-08-10-performance-frontier.md)).
Beating the per-entry stat floor therefore requires an operation log, and macOS has one
on disk already: `fseventsd` journals directory-level change events persistently, across
process exits and reboots.
Storing the journal cursor in the snapshot and replaying “what changed since” turns a
quiet tree’s revalidation from ~700 ms at 60k entries into an approximately
size-independent tens of milliseconds — against today’s serial sweep; the Background
section states the honest comparison against where rung-1 work will land, and where
the journal is transformative rather than incremental.

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

- Persist an FSEvents journal cursor (platform tag, volume UUID, event ID, capture time)
  in the snapshot, on macOS, at save time
- On load, when a strict gate passes, replay the journal since the cursor and revalidate
  only the named directories, emitting ordinary conditional deltas
- Map every journal degradation signal onto the existing escalation vocabulary: scoped
  `InvalidateSubtree` where the flag is scoped, full sweep where it is not
- Keep correctness identical to the sweep: same oracle, same digests, verified per trial
  by the performance harness on both paths
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
be sound. Only change information can — and `fseventsd` records it: directory-granular
events, persisted to disk in per-volume journals, addressed by IDs from a machine-wide
monotonic counter, replayable from a stored ID via `FSEventStreamCreate(sinceWhen:)`,
with explicit flags for every way the history can be insufficient
(`kFSEventStreamEventFlagMustScanSubDirs`, `UserDropped`, `KernelDropped`,
`EventIdsWrapped`, `HistoryDone`, `RootChanged`, `Mount`, `Unmount`).

Crucially, the journal’s directory granularity carries exactly the information directory
mtimes do not: a content edit to `a/b/c/file.txt` produces an event naming `a/b/c`,
because `fseventsd` logs operations rather than namespace timestamps.
That is what makes subtree skipping sound here and unsound when inferred from mtimes.

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

Where it is transformative is everywhere the sweep cannot be fast: cold metadata caches
(cloud hosts whose RAM cannot hold the inodes — the snapshot is the only warm state
there), network storage (each avoided stat is a saved round trip against an IOPS
budget), and very large local trees — the research’s whole-drive scenario (H45: 2–5M
entries, where dust takes 30–60 minutes and the journal plus persisted roll-ups target a
seconds-scale recheck).
At drive scale the journal is not an optimization of the sweep; it is the difference
between a tool that can be part of a working loop and one that cannot.

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

- `sinceWhen` delivers events with IDs *strictly greater than* the stored ID, so the
  cursor is `FSEventsGetCurrentEventId()` captured at save time.
- The end of history is a sentinel event flagged `HistoryDone` whose path is meaningless
  and must be ignored.
- Event IDs are allocated from a machine-wide monotonic counter, but the journals they
  index — and their retention — are per-volume.
  Both facts drive the gate: G4 compares the cursor against the machine-wide current ID,
  and the volume UUID (from `FSEventsCopyUUIDForDevice(st_dev)`, never `st_dev` itself,
  which is not stable across reboots) pins which volume’s journal the cursor belongs to.
  Wrap is signalled by `EventIdsWrapped` (G7).
- Stream `latency` may be 0 for replay; coalescing within the historical log has already
  happened.
- Callbacks arrive on the dispatch queue with C types; the callback marshals into a
  plain `Vec<(PathBuf, flags)>` behind a channel and does nothing else — all logic stays
  in safe code on the calling thread.

## Design

### Where it fits

The architecture invariant survives untouched: **the journal is a producer of scope,
never of state.** Nothing new mutates the index.

```
snapshot.load ──► index          (as today)
      │
      ▼
journal gate ──► pass ──► replay since cursor ──► changed-dir set
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
                                     snapshot.save with new cursor
```

The scoped revalidation reuses the sweep’s own emission logic bounded to a directory
set: for each named directory, re-list it, stat its immediate children, emit conditional
upserts and removals against the index’s recorded expectations — the same ops the full
sweep would have emitted for those directories, with the same ABA arbitration on apply.
A `MustScanSubDirs` flag on a path becomes `InvalidateSubtree(path)` resolved by the
existing subtree reconcile; the escalation vocabulary already fits because the watch
layer needed it first.

### The gate

The user-visible rule is “journal when provably applicable, sweep otherwise,” and the
gate is a pure decision function so the whole table is unit-testable without
CoreServices. Every row falls closed to the sweep:

| # | Condition | Decision |
| --- | --- | --- |
| G1 | Not macOS, feature off, or `--revalidate=full` | full sweep |
| G2 | Snapshot has no cursor (older format, or first save) | full sweep; write cursor on save |
| G3 | Root’s current volume UUID ≠ stored UUID (moved disk, container change, UUID unreadable) | full sweep |
| G4 | Stored event ID > current volume event ID (regression: journal purged, clock wrapped) | full sweep |
| G5 | Snapshot older than `max_journal_age` (default 14 days) | full sweep — paranoia bound; Apple documents the journal as advisory |
| G6 | Stream creation fails or replay exceeds `replay_timeout` (default 5 s) without `HistoryDone` | full sweep |
| G7 | Replay reports `EventIdsWrapped`, `RootChanged`, `Mount`, `Unmount`, `UserDropped`, or `KernelDropped` | full sweep |
| G8 | Replay reports `MustScanSubDirs(path)` | scoped: `InvalidateSubtree(path)`, journal continues for the rest |
| G9 | Changed-dir set exceeds `max_changed_fraction` (default 25%) of the snapshot’s directories | full sweep — scoped work would approach sweep cost with worse locality |
| G10 | Otherwise | scoped revalidation of the changed-dir set |

The changed-dir set is normalized before G9: paths outside the root are dropped,
descendants of a `MustScanSubDirs` subtree are absorbed into it, duplicates coalesce,
and paths are mapped root-relative against the same canonicalized root the scan layer
uses.

Volume identity is the UUID, not `st_dev` — device numbers are not stable across
reboots. The research doc’s sharding observation applies: journals, event IDs, and UUIDs
are per-volume, and a snapshot that spans a mount boundary cannot use a single cursor.
Phase 1 sidesteps this by combining the gate with the existing `one_filesystem` scope
information: a snapshot whose scan crossed devices simply never carries a cursor (G2).

### Snapshot format

`FORMAT_VERSION` 2 → 3. After the scope header, one new optional section:

```
journal_cursor: u8 tag        0 = none, 1 = fsevents-v1  (room for usn-v1 = 2)
if fsevents-v1:
  volume_uuid: 16 bytes
  event_id:    u64            FSEventsGetCurrentEventId() at save time
  captured_at: i64 ns         wall clock at save, for G5
```

The cursor is captured **before** the scan that populates the index begins, not after it
ends: events for mutations that race the scan then replay on the next open and are
re-verified, which double-checks work rather than losing it.
A v2 snapshot loads as cursor-absent (G2), not as invalid — the usual
corrupt-fails-closed rules are unchanged, and a corrupt cursor section discards the
snapshot like any other parse failure.

### Components

- `crates/fdu/src/journal/mod.rs` — platform-neutral surface: `JournalCursor`
  (encode/decode), `GateDecision`, the gate function, changed-set normalization.
  Compiles everywhere; no FFI.
- `crates/fdu/src/journal/fsevents.rs` — `#[cfg(target_os = "macos")]`, feature
  `journal`. The FFI module: current event ID, volume UUID for a device, and historical
  replay via the non-deprecated dispatch-queue API (create stream with `sinceWhen`,
  `FSEventStreamSetDispatchQueue` onto a private queue, start, receive marshalled
  `(path, flags)` pairs over a channel until `HistoryDone` or the G6 deadline, then
  stop/invalidate/release).
  The only module in the workspace allowed `unsafe`.
- `crates/fdu/src/scan.rs` — `revalidate_dirs(index, dirs, config, sink)`: the bounded
  sweep. Reuses the existing per-directory emission; no new op kinds.
- `crates/fdu/src/snapshot.rs` — format v3 fields, cursor capture on save.
- `crates/fdu/src/cli.rs` — wire the gate into the cached path; `--revalidate=auto|full`
  (default `auto`; `full` forces the sweep).
  `--no-cache` is untouched.
- Feature `journal` in `crates/fdu/Cargo.toml`: gates `dep:fsevent-sys` (macOS only via
  target-conditional dependency) and the FFI module.
  Off by default initially; the CLI enables it once the evidence is in.
  On non-macOS targets the feature compiles to the gate returning G1, so
  `--no-default-features` and Linux/Windows builds are unaffected.

### API changes

Additive only. `snapshot::save`/`load` signatures unchanged (cursor handled internally).
New public surface: `JournalCursor`, `GateDecision`, and `scan::revalidate_dirs`, all
documented as macOS-accelerator plumbing with the sweep as the portable contract.

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
- [ ] Replay semantics hold: IDs are strictly greater than `sinceWhen`; a content edit
  deep in the tree produces an event naming the file’s *parent directory* (the
  load-bearing granularity claim); deletions and renames appear; replay of an hour-old
  and a week-old cursor completes quickly
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
- [ ] Findings are recorded by amending this spec’s gates and constants (G5/G6 defaults,
  G9 fraction) and noted in the experiment ledger; explicit go/no-go for Phase 2

### Phase 1: Format and gate (mergeable alone; unblocks the block-format spike)

- [ ] Snapshot format v3: cursor section, save-side capture stub (writes `none` on all
  platforms), load-side decode, corrupt-cursor fails closed
- [ ] `journal/mod.rs`: cursor types, gate decision table as a pure function,
  changed-set normalization; exhaustive unit tests for every gate row
- [ ] Golden and round-trip tests: v2 loads as cursor-absent; v3 round-trips

### Phase 2: Replay and scoped revalidation (macOS)

- [ ] `journal/fsevents.rs`: FFI declarations, current-event-id, volume UUID, historical
  replay with deadline; scoped `#[allow(unsafe_code)]` with per-call safety comments
- [ ] `revalidate_dirs` in scan.rs, plus `InvalidateSubtree` resolution for G8
- [ ] CLI: gate wiring, `--revalidate` flag, save-side cursor capture on macOS
- [ ] Integration tests (macOS CI leg): mutate-then-journal-revalidate equals fresh scan
  by engine digest; UUID mismatch, event-ID regression, and forced `MustScanSubDirs`
  each degrade correctly
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
the same equality the parallel-walker tests pin.
Degradations are forced, not simulated: a wrong stored UUID, a stored event ID above
current, an undersized `max_changed_fraction`. The performance harness needs no changes
to verify correctness: its oracle already digests every trial’s index, so a journal-path
trial that skips a real change fails the run loudly.
Linux and Windows CI prove the feature compiles away cleanly.

## Rollout Plan

Feature `journal`, off by default, on for the CLI build once both phases pass the gate
*and* the loop’s experiments accept.
The README and skill text may only claim what the ledger shows, per the existing
no-unmeasured-claims convention.

## Open Questions

- Does `fseventsd` directory granularity hold under high-frequency mixed workloads (the
  research doc’s open question 3)? The churn experiment answers it empirically; G9
  bounds the damage if the answer is unfavorable.
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
