# Feature: FSEvents-Scoped Revalidation (macOS)

**Date:** 2026-08-10

**Author:** fdu project

**Status:** Proposed design, revised 2026-09-26; production implementation pending.
The August exploratory spike is complete, but its source and exact flags were not
retained. The [reproducible probe](../../../../explorations/fsevents-replay/README.md)
(`fdu-uwhl`) exercises immediate cross-process replay; long-gap and publication
boundaries remain acceptance work before production integration.
[Campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md) places the persistent
representation and replay work in Phase D; the probe can run independently.

Production replay is not implemented on `main`: the snapshot format has no replay
cursor, and `fdu-core` has no FSEvents replay module.
The [disk-usage checkpoint plan](plan-2026-09-13-fdu-disk-usage-checkpoints.md) supplies
the durable before/after comparison that this refresh mechanism alone does not provide.

**Terminology.** In this plan, *journal* means an operating system’s persistent change
journal: FSEvents history on macOS, or the USN journal on Windows.
It is unrelated to the engine’s index journal
([`opened/journal.rs`](../../../../crates/fdu-core/src/opened/journal.rs)), the bounded,
process-local commit history behind `since(clock)`. Proposed code uses history-replay
names (the `history_replay` module and `ReplayCursor`) so the two never share an
identifier. Native replay is proposed under the existing `watch` build feature; enabling
that build feature does not start a watcher or request replay.

## Overview

**fdu need not remain running between scans.** macOS’s system `fseventsd` maintains the
persistent history. fdu saves its inventory and cursor, exits, and later starts a new
process to request changes since that cursor.
Apple’s
[persistent event guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html#//apple_ref/doc/uid/TP40005289-CH4-SW6)
explicitly describes discovering changes while an application is not running and across
reboots. No fdu daemon, scheduled refresh, or Spotlight indexing is required.
The availability and completeness of retained history still limit this route.

Reduce the filesystem work of a warm start on macOS by replaying a device-relative
FSEvents stream from the last boundary whose work was applied, and revalidating only the
normalized scopes its events name.

The snapshot revalidation path loads the snapshot, builds the index, then walks the
entire tree again to observe its current state.
Current one-shot metadata execution can bypass that load and scan directly.
The full sweep is the only sound revalidation the filesystem itself offers, because
change information does not propagate upward through directory mtimes — an in-place file
edit is invisible to every ancestor directory, including its immediate parent (verified
empirically on APFS; see the
[change-propagation analysis](../../research/research-2026-08-10-performance-frontier.md)).
Beating the per-entry stat floor therefore requires an operation log, and macOS has one
on disk already: `fseventsd` journals change events persistently, across process exits
and reboots. Storing an applied journal boundary in the snapshot and replaying “what
changed since” can avoid most of that sweep.
How much it avoids is a hypothesis until the state machine below is implemented and
measured: no experiment in the ledger supports a size-independent latency claim for this
path.
Scoped filesystem work depends on the entries in dirty scopes, while replay depends
on retained history and volume traffic.
The current flat snapshot still incurs O(tree) loading when nothing changed; when a save
is needed, it rewrites the full image.
The Background section replaces that original serial comparison with exp-030’s current
bounded-parallel rung-1 baseline and states where journal refresh could help.

This is rung 2 of the warm ladder in the
[performance-frontier research](../../research/research-2026-08-10-performance-frontier.md)
(backlog item H43). One honesty note that research’s source-level read established and
this plan inherits: Watchman’s `fsevents_try_resync` proves the *mechanics* (resume from
a recorded event ID, UUID-guarded, wrap-vetoed) but uses them only for in-process
recovery, off by default — and across restarts both Watchman and git’s fsmonitor daemon
start at `SinceNow` and re-crawl.
These reviewed restart paths do not demonstrate one-shot inventory resume.
The
[September prior-art update](../../research/research-2026-09-27-persistent-change-prior-art.md)
adds CCC Quick Update and SuperDuper Turbo: production FSEvents-guided enumeration
between backup jobs, but without published cursor/barrier implementations.
fdu must still establish its own correctness and cost.
That is why the validation spike is Phase 0 rather than an afterthought, why the gate
fails closed on every row, and why the full sweep remains the backstop on every platform
— the route required when the caller requests full verification.

## Goals

- Persist an FSEvents journal boundary (platform tag, volume UUID, applied event ID,
  capture time) in the snapshot, on macOS, taking the fence *before* the scan and
  storing only the boundary whose work reached the index
- On load, when a strict gate passes, replay a device-relative stream since that
  boundary, normalize the item events it delivers into directory scopes, and revalidate
  only those scopes, emitting ordinary conditional deltas
- Map every journal degradation signal onto the existing escalation vocabulary: scoped
  `InvalidateSubtree` where the flag is scoped, full sweep where it is not
- Require the same facts as the sweep on every controlled test fixture, compared by the
  independent oracle; preserve weaker provenance for journal-scoped answers even when
  those tests pass
- Land the numbers through the performance loop as experiments, with the accept rule
  deciding
- Choose the measured cheaper path automatically, subject to the caller’s freshness
  requirements and the gate.
  Journal-scoped results must retain their distinct provenance; a latency prediction
  cannot upgrade them to full verification.

## Non-Goals

- Windows (USN journal) and Linux adapters: this plan implements the macOS adapter.
  Ordinary Linux inotify/fanotify queues cannot replay an offline gap; optional resident
  observers and filesystem-specific change sources are separate work.
  Portable full scans preserve the same checkpoint workflow on every platform.
- Changing what is cached or where.
  This feature accelerates revalidation.
  The cursor rides in the existing snapshot, and the typed-gap rule that lets a
  gap-marked image persist belongs to the checkpoint plan.
  Durable checkpoints, comparison, and retention belong to the linked checkpoint plan.
  Current content-sidecar reuse and portable full sweeps remain available independently.
- Touching the live watch layer.
  The watcher (rung 3) already exists behind the `watch` build feature; this is the
  between-runs story. Sharing its build feature does not require a resident process.
- Spotlight or any other time-indexed query source.
  A query for currently indexed recent files cannot recover deleted paths or their old
  sizes. Comparing retained inventories can; an operation log accelerates their refresh.
- The block snapshot format (`fdu-pdra`, `fdu-yr23`). The cursor initially rides the
  flat format; bounded persistence is a separate prerequisite for the large-tree
  end-to-end latency target.
- Trusting the journal.
  Apple documents FSEvents as advisory.
  It nominates where to look.
  Changed facts come from fresh observations; untouched facts remain retained
  observations with journal-scoped provenance.

## Background

The plan’s original baseline was measured on a 59,654-entry real checkout (Apple M1 Pro,
APFS, warm page cache):

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
be sound. Only change information can — and `fseventsd` records it, persisted to disk in
per-volume journals.
The stream, its UUID, and retained history are bound to a volume.
Software that persists a cursor across launches should create a per-device stream with
`FSEventStreamCreateRelativeToDevice`. A stored ID is replayed as `sinceWhen` in that
stream’s domain, with flags for *several* of the ways history can be insufficient
(`kFSEventStreamEventFlagMustScanSubDirs`, `UserDropped`, `KernelDropped`,
`EventIdsWrapped`, `HistoryDone`, `RootChanged`, `Mount`, `Unmount`).

The iterative loop has since improved those constants without changing that shape.
On the current 60,067-entry subject, exp-032 measures about 290 ms for cold index and
207 ms for snapshot load.
exp-026 wires the exp-022 `getattrlistbulk` reader into full reconciliation, and exp-030
then compares bounded directory waves against an immutable baseline so exact no-ops
never reach the index consumer.
Warm-open wall is now about 351 ms at 60k and 5.71 seconds at 720k; exp-030 alone
improves those paths 30.25% and 59.53%. This improves the full-sweep fallback without
changing its O(tree) verification shape.
After the composable CLI merge, exp-035 reproduced the current branch on the
heterogeneous 1,007,659-entry workspace: cold indexed wall is 7.332 seconds versus 9.850
seconds on merged `origin/main`, with exact digest parity.
That is not a journal result, but it is the current live non-cached scale anchor the
journal fallback must preserve.

**Degradation flags do not prove completeness.** The historical spike reported missing
expected events from an old cursor without a warning.
Its exact flags were not retained, so it does not distinguish expired history from
boundary behavior without `FullHistory`. Apple documents FSEvents as advisory;
`HistoryDone` only marks the end of delivered history.
The gate and the engine’s `Source::JournalScoped` provenance preserve that limit.

The journal does carry exactly the information directory mtimes do not: a content edit
to `a/b/c/file.txt` produces an event, because `fseventsd` logs operations rather than
namespace timestamps.
What it does not carry is a promise about *granularity*. This design requests
`kFSEventStreamCreateFlagFileEvents`, so a callback names a filesystem item rather than
a changed directory, and coalescing may report an item and its parent in either order.
Soundness therefore comes from normalization, not from the callback shape: a file or
symlink event becomes a relist of its parent directory, a directory create, remove, or
rename becomes a parent relist plus a subtree invalidation when the directory still
exists, and any item flag combination the normalizer cannot resolve falls back to the
full sweep. That normalization is what makes subtree skipping *informative* here and
useless when inferred from mtimes — a strictly better signal, but still not a sound one,
because being told the truth about what changed is not the same as being told the whole
truth.

### Where Journal Refresh Could Help

H26’s bulk reader and H12’s bounded parallel no-op elision brought full reconciliation
to about 151 ms and the complete warm open to about 351 ms at 60k entries (exp-030). The
historical replay timings below can consume much of that budget themselves.
There is no accepted whole-command journal result against this improved baseline.

The stronger target is a large supported local volume where a full traversal dominates
latency. H45 proposes a seconds-scale recheck using journal discovery and persisted
roll-ups; this remains a target to measure.
Do not assume a local macOS journal covers remote filesystem changes.
Unsupported storage must use the scan fallback.

The sweep must stay fast because journal refresh degrades into it.
Phase 2 compares against the current full-scan and revalidation paths, with snapshot
loading, publication, and fallback overhead included.

Scoped revalidation also composes with the platform work rather than competing with it.
Both the cold-scan and full-reconciliation halves of H26 have landed.
The remaining integration is orchestration: feed journal-named changed directories into
the existing bulk-backed subtree reconciler.
The research’s whole-drive composition remains journal resume (H43) naming the
directories, bulk re-scan (H26) verifying them, and persisted roll-ups (H33/H16)
rendering the rest untouched.

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
| `fsevent-sys 4.1.0` (already present via `notify`) + own declarations | **0** | Ships `FSEventStreamCreate`, start/stop/invalidate/release, `FSEventsGetCurrentEventId`, and every event flag. Leaves `FSEventStreamCreateRelativeToDevice`, `FSEventStreamSetDispatchQueue`, and `FSEventsCopyUUIDForDevice` commented out; all three are declared in our FFI module, alongside `dispatch_queue_create`/`dispatch_release` (libdispatch is part of libSystem, linked on every macOS binary — no crate needed). |
| `objc2-core-services` + `objc2-core-foundation` + `dispatch2` | ~4–5 | The modern generated bindings (verified: all four needed functions exist, including `FSEventStreamSetDispatchQueue` and `FSEventsCopyUUIDForDevice`). Signatures machine-derived from Apple headers. |

**August preference: option 1; September decision reopened.** Reusing the locked binding
avoids new packages, but the newer `fsevent-sys 5.2.0` is deprecated in favor of
`objc2-core-services`, and notify’s 9.0 RC line has moved to objc2. Before engine
integration, compare the maintained generated route against the locked binding plus
reviewed declarations, including API coverage, unsafe ownership, MSRV, build-feature
isolation, and supply-chain review.
The table records the earlier dependency estimate, not a newly resolved lockfile.
The standalone SDK probe settles API behavior without deciding Rust dependencies.

The workspace denies `unsafe_code`; this FFI module carries a scoped
`#[allow(unsafe_code)]` with every call site documented.
The exp-022/026 `getattrlistbulk` work has established the same pattern for the scan
boundary: an exact already-locked binding, unsafe confined to one leaf module, and
byte-for-byte portable parity tests.
The FSEvents module remains behind `watch`, which is non-default in `fdu-core`, and a
macOS platform gate.
Runtime replay remains an explicit request.

Replay semantics that the implementation and its tests must honor, from Apple’s
documentation and Watchman’s source (mechanics only — see the Overview’s honesty note on
how far its production use actually goes):

- With `FullHistory` (macOS 10.15+), the first historical chunk can include events at or
  below `sinceWhen`. Apple documents this overlap as protection against events missed at
  the boundary. Do not filter it away: repeated observations must be idempotent.
  What is persisted must be a boundary whose work is already in the index — never one
  sampled later. Capturing `FSEventsGetCurrentEventId()` at save time gets this
  backwards: the walk ran *before* the save, so a change made during the walk and missed
  by it sits below the saved cursor and is never replayed.
  The fence is therefore taken *before* the scan begins, and a replay advances the
  stored boundary only as far as the events it actually applied.
  Re-scanning a little on the next open is the safe direction; skipping a change is not.
- The end of history is a sentinel event flagged `HistoryDone` whose path is meaningless
  and must be ignored.
- Persisted replay uses per-device stream IDs.
  Apple’s guide distinguishes their ordering from per-host IDs, so the probe must
  establish comparable capture and completion boundaries rather than mixing ID domains.
  G4 rejects an invalid boundary, and the volume UUID (from
  `FSEventsCopyUUIDForDevice(st_dev)`, never `st_dev` itself, which is not stable across
  reboots) pins which volume’s journal the cursor belongs to.
  That UUID identifies the volume’s FSEvents database and changes when the database is
  discarded or event IDs wrap, which is what G3 must detect.
  It is not the filesystem volume UUID (`ATTR_VOL_UUID`) that the checkpoint plan
  records as a checkpoint’s volume identity, which must survive a history purge.
  Wrap is signalled by `EventIdsWrapped` (G7).
- Stream `latency` may be 0 for replay; coalescing within the historical log has already
  happened.
- Callbacks arrive on the dispatch queue with C types; the callback marshals into a
  plain collection of `(path, flags, event_id)` records behind a channel — all logic
  stays in safe code on the calling thread.

## Design

### Cross-process lifecycle

The first implementation supports a local macOS root with `one_filesystem` scope.
Every run uses the same root, stored-state identities, and managed-store exclusion
policy. The sequence is:

1. Capture a device-relative journal fence before the initial full scan.
   Record its journal database UUID and capture time.
   Publish the completed inventory with that conservative fence, then exit.
2. Leave fdu stopped while the user edits, installs, deletes, or moves files.
   The operating system records available events independently of fdu.
3. On the next invocation, admit the snapshot through the existing scope and engine
   identity checks, then apply the replay gate below.
4. Replay into bounded owned records, accepting `FullHistory` overlap.
   Normalize events into dirty scopes, verify them with the existing scanner, and apply
   the observations through the engine’s exact commit path.
5. Publish the updated inventory, trust state, and fully applied replay boundary as one
   revision. A subsequent process resumes from that published boundary.

Cursor capture does not turn a traversal into a point-in-time filesystem snapshot.
Changes racing the scan remain eligible for the next replay.
If an opened lifecycle promises to enter watching with a closed registration gap, it
must additionally use the existing observation-before-discovery and
buffered-reconciliation protocol.
That live handoff is separate from the first one-shot replay implementation.

### Verification policy and user-visible trust

The proposed policy has two explicit choices, with names subject to API review:

| Requested verification | Permitted route | Meaning of the answer |
| --- | --- | --- |
| `full` (default) | Fresh scan or full reconciliation | Filesystem observations over the requested scope, with ordinary partial/error reporting |
| `journal` (explicit opt-in) | Replay and scoped reconciliation, or full fallback | Untouched facts depend on journal completeness and carry `Source::JournalScoped` |

`--cache auto` continues to describe storage policy; it does not authorize weaker
verification. No entry-count threshold or latency estimate may change the requested
verification. The engine planner chooses the cheaper route only among those the request
permits. Cache-only remains its separate, explicitly stale contract.

This is a proposed extension to the set of accepted answer guarantees.
Before a public release, the request model and path-independence contract must express
it explicitly (`fdu-kfp6`). The current default’s cold-answer invariant remains intact.
An explicit journal request can receive the stronger fully scanned answer when replay is
unavailable. No new CLI flag ships until Rust and Python can express the same choice.

Freshly observed entries retain their observation provenance; untouched contributors
remain journal-scoped, and a roll-up takes the weakest contributing trust.
Text output must visibly label a journal-scoped answer, and machine output must retain
that label, coverage, observation times, last full-verification time, and any fallback
reason. A new replay does not make untouched facts newly observed.
Snapshots must preserve enough trust state that reloading cannot upgrade a journal
result into verified data.

Mixed-source progressive serving is deferred.
A later interactive client may display a provisional journal answer and request a
background sweep, but only after per-subtree trust, deletion, and convergence obey the
engine architecture’s composition rules.

### Managed stores and partial coverage

Whole-home roots include fdu’s default cache directory.
Writing an inventory inside the observed tree can otherwise schedule another refresh and
count fdu’s own growth.
The proposal is an explicit engine-owned managed-store exclusion in scan scope, recorded
in stored-state identity and shared by cold scans, full reconciliation, replay, and
checkpoints (`fdu-gmw2`). Report those store bytes separately.
A view-time exclude or an FSEvents self-event flag cannot establish that common scope.
The disk-history profile opts into this scope explicitly; it must not silently alter
ordinary roll-ups or cached content-analysis requests.

Inaccessible subtrees are unknown, not deleted or unchanged.
A persistent gap must be recorded before its event can count as applied.
Until the snapshot can encode typed gaps, a partial pass cannot advance the durable
cursor or replace its complete baseline.
The checkpoint plan owns gap persistence; the first production slice can be restricted
to complete readable roots.
Long-lived partial roots require that gap slice.

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
                                     engine exact Commit path
                                              │
                                              ▼
                                     snapshot.save persists the pre-scan
                                     or replay-advanced cursor
```

The scoped revalidation reuses the sweep’s own emission logic bounded to a directory
set: for each named directory, re-list it, stat its immediate children, emit conditional
upserts and removals against the index’s recorded expectations — the same ops the full
sweep would have emitted for those directories, with the same ABA arbitration on apply.
A `MustScanSubDirs` flag on a path becomes `InvalidateSubtree(path)` resolved by the
existing subtree reconcile; the escalation vocabulary already fits because the watch
layer needed it first.

### The gate

The user-visible rule is **“journal only when explicitly permitted and the gate passes;
sweep otherwise.”**

The word *provably* does not belong here, and an earlier draft of this plan used it.
The Phase 0 report raised the opposite concern: an old `sinceWhen` returned
`HistoryDone` without the expected history or a degradation flag.
The committed probe must reproduce this with known fences and recorded flags, including
`FullHistory`. Scoped revalidation stats the paths the journal *names* and does not stat
the rest, so the untouched majority of the tree is accepted on the journal’s
completeness — the one property that neither UUID equality nor `HistoryDone`
establishes. That is not a sound verification path, and calling it one would contradict
the project rule that a platform journal narrows what must be checked but never replaces
the checking.

So this plan takes the **risk-bounded** contract deliberately, and the honesty is
carried in the type rather than in prose: values the journal vouched for read as
`Source::JournalScoped`, never as verified, and a consumer that needs certainty can see
the difference on every row.
G5 and G12 below are **risk controls that bound how long a silent omission can persist**
— they are not correctness gates, and no row of this table proves any individual journal
answer right.

The gate is a pure decision function so the whole table is unit-testable without
CoreServices. Every row falls closed to the sweep:

| # | Condition | Decision |
| --- | --- | --- |
| G1 | Full verification requested, not macOS, `watch` build feature off, or unsupported storage | full sweep |
| G2 | No usable snapshot, or one without a cursor (first save, older format, or an image discarded by a release upgrade, a scope change, or `--cache-clear`) | full sweep; persist its pre-scan cursor with the snapshot |
| G3 | Root’s current volume UUID ≠ stored UUID (container change, FSEvents database purged or replaced, disk erased, UUID unreadable), or the root’s device number ≠ the one the snapshot recorded | full sweep; a renumbered device changes every retained `Fingerprint`, and a scoped refresh would leave entries under two device numbers |
| G4 | Stored event ID exceeds a comparable current journal boundary, or that comparison cannot be established | full sweep; never compare unrelated volume/host ID domains |
| G5 | Applied cursor older than `max_cursor_age` (provisional default **24 hours**) | full sweep; an age limit bounds exposure but does not prove retained history is complete |
| G6 | Stream creation fails, or replay exceeds the G11 budget without `HistoryDone` | full sweep |
| G7 | Replay reports `EventIdsWrapped`, `RootChanged`, `Mount`, `Unmount`, `UserDropped`, or `KernelDropped` | full sweep |
| G8 | Replay reports `MustScanSubDirs(path)` | scoped: `InvalidateSubtree(path)`, journal continues for the rest |
| G9 | Changed-dir set exceeds `max_changed_fraction` (provisional 25%) or bounded event storage is exhausted | full sweep; measure entries in dirty scopes as well as directory count |
| G10 | Otherwise | scoped revalidation of the changed-dir set |
| G11 | Replay wall exceeds a budget scaled to the estimated sweep cost (measured: replay runs ~200 ms typical, up to 2 s from an old cursor) | abandon replay, full sweep |
| G12 | Warm-open count (provisional 20) or an elapsed-time verification limit is reached | full sweep; persist both counters independently of replay-cursor age |

The cursor age, last full-verification time, and checkpoint age are separate values.
A week-old checkpoint can be compared against a freshly maintained inventory.
Conversely, a first refresh after 24 hours currently hits G5: this is an unresolved
daily workflow constraint, not evidence that next-day refresh is already fast.
Phase 0 must test 1-hour, 24-hour, 48-hour, and 7-day gaps before changing either risk
control.

The changed-dir set is normalized before G9. Unrelated paths outside the root are
dropped, but an ancestor scope that covers the root must invalidate the root when its
flags require recursive reconciliation.
Global degradation flags are processed before path filtering.
Descendants of a `MustScanSubDirs` subtree are absorbed into it, duplicates coalesce,
and paths are mapped root-relative against the scan layer’s root.
Keep stream progress separate from this filtered work set.

The cursor’s volume identity is that FSEvents database UUID, not `st_dev` — device
numbers are not stable across reboots.
The research doc’s sharding observation applies: journals, event IDs, and UUIDs are
per-volume, and a snapshot that spans a mount boundary cannot use a single cursor.
Phase 1 sidesteps this by combining the gate with the existing `one_filesystem` scope
information: a snapshot whose scan crossed devices simply never carries a cursor (G2).

### Snapshot format

The snapshot format `main` writes, at the current `snapshot::FORMAT_VERSION`, has no
replay cursor. Use the next available format version at integration time, and do not
assign one version to incompatible layouts.
After the scope header, propose one new optional section:

```
replay_cursor: u8 tag         0 = none, 1 = fsevents-v1  (room for usn-v1 = 2)
if fsevents-v1:
  journal_uuid: 16 bytes      FSEvents database identity, not filesystem volume UUID
  event_id:    u64            applied fence; initially sampled before the full scan
  captured_at: i64 ns         time associated with that fence, for G5
replay_state:
  last_full_verification_started_at: i64 ns
  replay_opens_since_full: u64
  trust_and_gaps: versioned section
```

The cursor is captured **before** the scan that populates the index begins, not after it
ends: events for mutations that race the scan then replay on the next open and are
re-verified, which double-checks work rather than losing it.
Current loading rejects incompatible versions.
If the new reader supports a known older format explicitly, it must load it as
cursor-absent (G2); otherwise it remains a clean miss.
Test each supported predecessor, unknown version, and corrupt cursor.
Cursor, inventory generation, and publication must commit atomically: never save a
boundary ahead of the reconciled data.

These are proposed logical fields, not a frozen binary layout.
Persist G12’s state so restarting fdu cannot reset the full-verification obligation.
Validate all lengths, tags, counts, and timestamps before admitting the image.
A future or invalid capture time cannot make an old cursor young; if wall-clock changes
prevent an age decision, fall back.
The engine fingerprint and scope identity still govern the whole image.

A replay transaction needs a completed boundary in the stream’s ID domain, including
when no root-local event arrives.
The probe must establish how that boundary is obtained and ordered with `HistoryDone`;
the maximum matching path ID is insufficient.
Never sample a later global ID after reconciliation and treat it as applied progress.
If a safe boundary cannot be established, retain the old cursor or scan again.
Events arriving beyond the applied boundary remain eligible for the next replay.

Concurrent writers serialize or compare the inventory revision before publication.
Cancellation, queue overflow, reconciliation failure, or a failed save cannot publish a
new cursor alone. The commit point installs facts, trust, gaps, and cursor together; an
interruption leaves either the previous revision or the complete new one.
Specify file and directory synchronization for crash durability separately from atomic
rename visibility, and inject failures on both sides of that publication boundary.

The snapshot is the checkpoint plan’s working inventory, and the cursor stays in it on
purpose. Both are derived state that a full scan rebuilds, so they share the cache’s
VERSION + FAIL FAST contract and one publication boundary.
A release upgrade changes `engine_fingerprint`, which discards the image and its cursor
together, and the next open takes G2. Checkpoints live in that plan’s separate store and
are unaffected. Before a cursor may advance past a denied subtree, the snapshot also
needs the typed-gap section that plan’s slice 3 specifies.
With it, `save` accepts a `Coverage::Partial(Inaccessible)` index only when every
incomplete directory is recorded as a gap, and still refuses any other partial index.

### Components

- `crates/fdu-core/src/history_replay.rs` — private platform-neutral types:
  `ReplayCursor` (encode/decode), `GateDecision`, the gate function, changed-set
  normalization. Compiles everywhere; no FFI.
- `crates/fdu-core/src/history_replay/fsevents.rs` — `#[cfg(target_os = "macos")]`,
  build feature `watch`. The FFI module: comparable event boundary, journal UUID for a
  device, and historical replay via the non-deprecated dispatch-queue API (create stream
  with `sinceWhen`, `FSEventStreamSetDispatchQueue` onto a private queue, start, receive
  marshalled `(path, flags, event_id)` records over a channel until a validated
  completed boundary or the G6 deadline, then stop/invalidate/release).
  `HistoryDone` alone is not yet established as that boundary: the probe also flushes
  buffered contemporary events, but does not prove that protocol sufficient.
  Watchman reports earlier changes arriving after both a cookie notification and
  `FSEventStreamFlushSync` under load; see its
  [synchronization limitation](https://facebook.github.io/watchman/docs/cookies#limitation-macos-fsevents).
  Quiescence and matching finite oracles do not establish a universal barrier.
  Specify the completion and queue-drain protocol only after testing changes pending at
  invocation and racing the history/live transition; a deadline without a proven
  boundary takes the conservative fallback.
  Unsafe is confined to this leaf module, following the existing scan-boundary pattern.
- `crates/fdu-core/src/scan.rs` — `revalidate_dirs(index, dirs, config, sink)`: the
  bounded sweep. Reuses the existing per-directory emission; no new op kinds.
- `crates/fdu-core/src/snapshot.rs` — versioned fields; encode and decode the cursor
  supplied by the caller.
- `crates/fdu-core/src/lib.rs` and the refresh orchestration — own the gate, pre-scan
  fence, reconciliation, and durable publication.
  CLI and Python delegate to the same engine capability.
  The CLI may expose `--revalidate=full|journal` after the engine policy exists (`full`
  is the default). `--cache off` remains the explicit full-scan, no-snapshot policy and
  bypasses the history-replay path unchanged.
- Existing build feature `watch` in `crates/fdu-core/Cargo.toml`: also gates the macOS
  replay FFI and the selected target-conditional optional native bindings.
  Even reusing a locked dependency requires normal dependency review.
  The optional native-observation capability remains removable under
  `--no-default-features`.

### API changes

Expose the verification choice and its outcome through the shared request, planner, and
report models. Keep `ReplayCursor`, gate internals, and scope reconciliation private
unless a concrete external caller requires them.
The index carries the applied cursor and trust state for snapshot save/load; library
callers must not assert their own verified cursor.
Opened refresh remains owned by `OpenedIndex`, using its existing commit and
cancellation boundaries rather than a parallel replay service.

### Packaging and platform fallback

The proposed packaging reuses `watch`, the engine’s only optional-capability build
feature (`fdu-5l1j`). It groups the existing live observer and historical replay as
native change discovery.
Compile-time inclusion and process lifetime are independent: a one-shot replay starts
only a temporary stream and stops it before returning.

- Without `watch`, or on unsupported platforms, the gate chooses the portable sweep.
  Enabling `watch` does not opt a request into journal-scoped trust.
- Native replay dependencies are optional and target-conditional under
  `[target.'cfg(target_os = "macos")'.dependencies]`; select their exact versions after
  the binding review above.
  Linux and Windows acquire no replay-specific native dependency.
- **Cargo consumers**: `default-features = false` builds keep the sweep path; consumers
  needing historical replay enable `watch` and explicitly request journal verification.
- **PyPI / uv consumers**: `fdu-py` wheels are built per-platform by maturin, so the
  macOS wheels carry the replay path and the manylinux wheels carry the fallback, from
  the same source with no Python-side conditionals, extras, or environment markers.
  `uv pip install fdu` (or `uvx fdu`) gets the right behavior on either OS because the
  platform selection already happened at wheel-build time — the same mechanism that
  ships every other platform difference today.
- Both distribution channels are exercised in CI: the existing test matrix
  (ubuntu/macos/windows) proves the build feature compiles and falls back everywhere,
  and the wheel legs install the built wheel and run the warm-path smoke on each OS.

## Implementation Plan

Sequencing follows the research’s optimization ladder rather than jumping it.
That ladder places journal resume in its final rung — every accelerator needs the lower
rungs as its fallback — while its measurement principle is “measure early, implement in
ladder order,” and it explicitly recommends reserving the snapshot fields now.
So the cheap, information-producing phases run immediately (the spike measures, the
format phase reserves), and the full implementation lands when the loop’s rung-1 warm
work (H12/H14) has produced the clean baseline Phase 2 must be judged against.

### Phase 0: Reproducible Journal Probe (macOS)

Commit the probe and its invocation to the repository’s exploration tooling before
making new claims. Record source revision, OS and filesystem, flags, stream kind,
latency, saved fences, event IDs, mutations, and full-scan comparisons.
It remains outside the shipped API. The standalone instrument uses the installed Apple
SDK directly, without changing fdu dependencies; Rust integration follows the binding
review above. The earlier scratch spike is historical evidence, not a reproducible
acceptance run. On a real volume, establish:

- [ ] Dispatch-queue delivery works as designed: stream created with `sinceWhen`,
  `FSEventStreamSetDispatchQueue` onto a private queue, events arrive, the `HistoryDone`
  sentinel arrives, teardown is clean, and a deadline abort works (gate G6’s mechanism)
- [ ] Replay semantics: compare runs with and without `FullHistory`; accept overlapping
  IDs, normalize actual file/directory events, and verify deep edits, deletion, rename,
  and subtree cloning against fresh scans
- [ ] Quiet-root progress: establish a safe completed replay boundary even when no
  matching root event arrives.
  Filtering events outside the root must not confuse per-volume replay progress with the
  largest root-local event ID
- [ ] Retention and degradation: replay known saved cursors after 1 hour, 24 hours, 48
  hours, and 7 days; record missing expected events even when no warning is emitted.
  Ancient and future synthetic IDs are negative controls, not retention measurements
- [ ] Volume identity works: `FSEventsCopyUUIDForDevice` for the root’s current `st_dev`
  returns a stable UUID across remount; the self-declared extern links.
  The SDK header says the UUID is stored on the device and travels with it to other
  computers, so G3’s UUID comparison does not catch a disk changed on another Mac and
  brought back: record whether replay from a cursor saved before the move names those
  changes, and which gate falls closed if it does not
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

### Selecting a Refresh Path

Current one-shot metadata reports can skip snapshot reads because a fresh scan avoids
paying for both reconstruction and a full sweep.
A retained inventory has a different job: it preserves before-state, supports content
reuse, and can support fixed-baseline comparison.
Its refresh decision must include those costs and semantics.

The August measurements suggest that journal replay has more room to help large or
slow-to-stat trees than small warm project trees.
They do not establish a universal entry-count threshold.
Metadata-cache pressure, directory shape, storage, competing work, and volume-wide
journal traffic all matter.
A vnode-capacity reading is a possible explanatory measurement, not proof that a
particular tree is cold or that the observed performance knee has been explained.

Compare measured end-to-end alternatives:

- Fresh inventory construction, preserving any pinned before-state needed for a delta.
- Loaded inventory plus a full sweep, including any content-sidecar reuse.
- Loaded inventory plus journal replay, scoped reconciliation, and durable publication.

The planner may use recorded scan and load costs, replay history, dirty-scope size, and
requested reducers to predict the cheaper permitted path.
G11 caps additional replay work, but an eventual fallback still pays the work already
attempted. Include that penalty in benchmarks; a timeout is not a guarantee of matching
fresh-scan latency. When no persistent journal is available, comparison remains possible
by retaining the baseline and scanning the current state.
The initial inventory always needs a scan.

For millions of entries, reducing filesystem calls is only one part of the target.
The flat image still loads and saves O(tree) records.
Persisted aggregates, lazy blocks, and bounded changed-block publication are
dependencies of the fast daily workflow in the
[checkpoint plan](plan-2026-09-13-fdu-disk-usage-checkpoints.md).

### The first scan is the other half of the problem, and the journal cannot help it

Measuring a real home folder made the large-tree regime concrete, and turned up
something the journal work does not address at all.

```
/Users/…  4,366,510 files, 1,016,449 dirs, 224 GiB
  791 s wall   15 s user   160 s system   2.65 GB peak RSS
```

Thirteen minutes, and the breakdown is the finding: **175 s of CPU against 791 s of wall
— 78% of the run was spent blocked, and the achieved parallelism was 0.22×.** Six worker
threads sat waiting on the SSD, 923,000 voluntary context switches deep.
Memory landed at ~493 B/entry, matching the frontier research’s ~490 B/entry estimate
almost exactly.

The automatic worker cap is six, and it was chosen honestly: exp-001 measured 2, 4, 6
and 8 threads on a warm 60k tree and found the knee at four, with eight *worse* than
four. That verdict is correct and it does not generalize one step outside the state it
was measured in. On a cold subtree of ~795k entries — large enough that the vnode cache
cannot hold it — interleaved runs say the opposite:

| workers | median | best |
| ---: | ---: | ---: |
| 6 | 33.7 s | 16.3 s |
| 16 | **17.0 s** | 12.4 s |
| 32 | 19.9 s | 14.3 s |

Roughly **2× from raising in-flight depth alone**, on the exact workload the whole-drive
use case cares about.
(The variance is wide because the subtree is live and the machine was not quiet; the
direction is unambiguous and consistent across all four rounds.)
`MAX_SCAN_THREADS` was 32, so the initial calibration included that upper bound rather
than assuming sixteen was the knee.

The post-breadth-first campaign turned that private observation into reproducible
evidence. Exp-015 built an immutable 720,805-entry cache-pressure subject from twelve
APFS clones of the pinned 60k tree.
On that subject, explicit sixteen workers improved end-to-end cold-index wall 11.72%
[−16.83%, −2.42%] against six; on the original 60,067-entry tree they instead regressed
it 5.64% [+2.08%, +7.96%]. Thirty-two did not improve on sixteen in calibration.
The old result survived the scheduler change, but it also confirmed that no fixed count
is right for both regimes.

Exp-018 through exp-021 then tested the shipped adaptive path.
Pre-creating dormant reserves added small-tree CPU, faults, and RSS, so it was rejected.
Creating them after 100,000 observed entries passed the 60k and 720k endpoints, but a
post-review 120k boundary run found no wall benefit and measurable RSS and fault
regressions.
Moving the trigger to metadata-cache capacity avoided that boundary but left
only a 1.71% unclear end-to-end gain at 720k.

The accepted selector measures the state directly.
Automatic scans aggregate the chunk work timing already collected for attribution and
make one decision after 16,384 entries.
At 30 microseconds of worker service per entry or more, one in-band control message
causes the consumer to create reserve workers, bounded by twice available parallelism
and sixteen overall.
There are no reserve threads, per-entry clocks, or polling before that point, and
explicit thread counts never adapt.
The 720k cold-index job improved 5.31% [−8.37%, −2.70%] and producer wall 10.09%; on the
120k boundary, wall, total CPU, faults, and RSS all remained unclear.
After activation, the large index job traded latency for 51% more aggregate CPU and
1.43% more RSS. These are warm-steady OS-cache measurements, not a controlled-cold
claim; the private roughly-2× observation remains motivating context.

The later bulk/BFS reproduction at 1M (exp-036) confirms why this remains adaptive
rather than a new high fixed default.
On the live APFS bulk path, eight workers improve wall only 1.30% while raising CPU
33.5%; twelve and sixteen regress wall 2.46% and 10.65%. The service-time trigger
correctly stays inactive in that fast state.
This does not reverse the portable high-latency result above; it proves the policy
distinguishes the two regimes it was designed for.

Two consequences, and the second is the important one:

- **Worker count must be state-adaptive, exactly like the cache policy.** When the tree
  fits the metadata cache the walk is syscall-bound and extra threads add contention;
  when it does not, the walk is latency-bound and extra threads buy throughput until the
  device saturates. A first scan has no recorded entry count, so it begins conservatively
  and calibrates from its own initial chunk service time.
  This direct signal adapts across storage states without waiting to discover that the
  tree is large.
- **This is worth more than journal resume for the motivating use case, and it is
  orthogonal to it.** A first scan of a home folder can never be helped by a journal:
  there is no cursor yet.
  Halving thirteen minutes is a bigger, simpler win than anything the journal does, and
  it lands on every platform rather than one.
  The journal’s job is the *second* scan; in-flight depth’s job is the first, and the
  product story for whole-drive usage needs both.

That reordered the work: the adaptive pool (`fdu-tt2j`) now precedes scoped
revalidation. It is portable and improves the case the journal is designed for without
depending on any of it.

### Phase 0 spike findings (2026-08-10, run on this host)

The following observations were recorded on 2026-08-10. The scratch source and exact
stream flags were not committed.
Preserve the observations, but reproduce them with the Phase 0 tool before using them to
set production policy.

**Confirmed.**

- *The load-bearing granularity claim.* An in-place append to a file at depth 17 in the
  59,654-entry reference tree changed **no** directory’s mtime and produced **exactly
  one** FSEvents event, naming the file’s parent directory.
  This is the whole basis of the feature: the journal carries what the namespace refuses
  to.
- *Changed-scope evidence.* Creating 20 files and 2 directories produced 7
  directory-level events; cloning a 257-entry, 24-directory subtree produced 47, every
  directory named, flagged `ItemCloned`. Coalescing is per directory, as documented.
- *The binding decision.* `fsevent-sys` from the existing lockfile plus six
  self-declared externs (`FSEventStreamSetDispatchQueue`, `FSEventsCopyUUIDForDevice`,
  `CFUUIDCreateString`, `dispatch_queue_create`/`_release`) compiles and links with zero
  new crates. The non-deprecated dispatch-queue path works: create, set queue, start,
  receive, `HistoryDone`, tear down.
  Volume UUID resolves from `st_dev`, and `HistoryDone` arrives with a meaningless path,
  as the plan assumed.

**Refuted: replay is not free, and its cost grows with cursor age.**

Empty replays on a quiet tree are bimodal — 17 trials split between ~9–33 ms and
~193–487 ms, with no warm-up trend.
Cost then grows with how far back the cursor reaches: roughly 150 ms at −1M event ids,
1.8 s at −20M, and 2.0 s at −94M.

This corrects this plan’s own headline.
“Tens of milliseconds at 60k” was optimistic: a warm open would be snapshot load plus a
replay that is often ~200 ms, against a full sweep of ~690 ms.
Real, but incremental — exactly the calibration the frontier research already argued
for, now measured rather than predicted.
Larger supported local trees may offer greater savings, but replay cost can grow with
cursor age and volume traffic, and flat-image loading and saving still grow with the
inventory. These timings are historical replay measurements, not daily-command results.

**Observed: old-cursor replay omitted expected events without a warning.**

Replaying the reference tree from `sinceWhen = 1` returned `HistoryDone` after one
event, with no `MustScanSubDirs`, `UserDropped`, or other recorded degradation flag.
The tree had been cloned hours earlier, so the experiment expected many more events.
A synthetic future cursor returned zero events and `HistoryDone` rather than an error.

The earlier interpretation was that history had been silently purged.
That remains a possible explanation, but the record does not establish it: there was no
known saved pre-mutation fence and the exact flags, including `FullHistory`, were not
retained. Apple’s SDK header documents that without `FullHistory`, events near
`sinceWhen` may be skipped; with it, the first historical chunk overlaps the cursor.
Neither that flag nor UUID equality restores expired history or proves complete
delivery. The engine’s `Source::JournalScoped` rustdoc still states the purge
interpretation as fact; `fdu-b9d5` aligns it with this record, using the committed
probe’s evidence (`fdu-uwhl`).

The 2026-08-10 response was to propose a 24-hour age bound, a replay deadline, and a
full sweep every 20 warm opens.
Keep those as provisional risk controls until the committed probe measures actual
elapsed-time boundaries and controlled mutations.
An age bound cannot establish correctness, and the current bound leaves the first
next-day refresh outside the fast path.

**Disposition:** proceed with a reproducible probe and the format/gate work; do not
claim size-independent warm opens or enable history replay by default from this record.
Full-scan oracle comparisons, replay-gap measurements, and whole-command cost are the
acceptance evidence still needed.

### Reproducible probe findings (2026-09-26)

The [instrument and records](../../../../explorations/fsevents-replay/README.md) provide
a reproducible starting point for further tests.
Preparation, mutation, and replay run in separate processes, with no application-owned
stream alive during mutation.
The candidate inventory is compared with an independent full `lstat` walk, and omitted
dirty scopes are a negative control.

Immediate quiet, mixed-mutation, and deep-append trials on APFS matched the oracle with
directory/file events and with/without `FullHistory`. The deep append changed a file
without changing any directory metadata.
These results support cross-process change discovery; they do not establish engine
accounting parity or journal completeness.

**Overlap can consume the expected savings.** `FullHistory` replayed fixture-creation
events at or below the saved cursor, even on a quiet fixture.
The probe’s conservative normalization selected the entire root in those trials.
Without `FullHistory`, the deep-append case selected its parent, but omitting that flag
is not an acceptable performance shortcut: Apple’s boundary-skip warning still applies.
Production normalization must distinguish an immediate directory relist from a recursive
invalidation, while preserving every required overlapping observation.
Measure the resulting work; do not infer a fast refresh from an oracle match that
rescanned the whole root.

The probe drains fixture construction and records the resulting stream ID before
scanning the baseline and exiting.
It retains the preliminary device-time fence and drain summary, because a more
conservative fence can include additional construction events.
The SDK’s `FSEventsGetLastEventIdForDeviceBeforeTime` takes Unix-epoch seconds despite
its `CFAbsoluteTime` type; using Core Foundation’s epoch would request the wrong time.

The instrument flushes contemporary events before stopping, but the immediate
observations do not establish a safe live handoff or advancing boundary under queued
mutations. Quiet-root cursor advancement, long elapsed gaps, reboot, volume moves,
controlled loss, and publication failures remain pending.
Proceed with the private gate and deterministic transaction design, not a public replay
release or a whole-command speed claim.

### Real-root and next-day findings (2026-09-27)

The
[extended probe and sanitized evidence](../../../../explorations/fsevents-replay/README.md#real-roots-and-next-day-replay-2026-09-27)
test existing roots without writing into them.
A quiet 12,280-entry root matched both full oracles with no candidate entry
observations. On a live 441,777-entry root, replay took about 181 ms, but the
conservative subtree normalizer rescanned the whole root.
Stable observations matched; concurrent mutations made overall validation inconclusive.
This is not yet evidence for the large-tree latency target.

The normalizer proposed above must implement immediate relists separately from recursive
invalidation; the prototype’s recursive parent scans are not acceptable as the
production fast path.
Whole-image persistence costs remain independently visible.

Replaying the preserved external APFS fixtures after about 26 hours produced events but
no `HistoryDone` before the ten-second deadline in all sixteen attempts.
These are recorded failures, not successful day-gap acceptance or proof of retention
loss.
Investigate bounded completion and volume-history cost before setting G5/G6 policy.
Phase 2 remains gated on this and the rest of the acceptance matrix.

### Phase 1: Format and gate (mergeable alone; unblocks the block-format spike)

- [ ] Next available snapshot format version: cursor section, encode-side cursor field
  stub (writes `none` on all platforms), load-side decode, corrupt-cursor fails closed
- [ ] `history_replay.rs`: private cursor types, gate decision table as a pure function,
  changed-set normalization; exhaustive unit tests for every gate row
- [ ] Round-trip tests for the new version; explicit predecessor migration or clean-miss
  tests, including the layout of the format version current at integration time
- [ ] Snapshot header records this tree’s observed scan cost (µs/entry) and entry count,
  so the cache carries its own cost model; a header without timing falls back to a
  conservative default
- [ ] Engine-owned refresh planning: compare measured full-scan and history-replay paths
  while preserving the baseline and respecting freshness policy; test decisions without
  OS APIs
- [ ] Model explicit journal acceptance, persist trust and full-verification history,
  and render the weaker provenance on every surface; default requests never take it
- [ ] Define managed-store exclusions as scan scope and test parity across routes
- [ ] Treat cache-capacity probes as optional performance diagnostics; do not use one
  host’s capacity as a correctness gate or universal threshold

### Phase 2: Replay and scoped revalidation (macOS)

- [ ] `history_replay/fsevents.rs`: FFI declarations, current-event-id, volume UUID,
  historical replay with deadline; scoped `#[allow(unsafe_code)]` with per-call safety
  comments
- [ ] `revalidate_dirs` orchestration in scan.rs, feeding changed roots into exp-026’s
  bulk-backed subtree reconciler, plus `InvalidateSubtree` resolution for G8
- [ ] Engine: gate wiring, capture the cursor immediately before a full scan, and commit
  only the applied fence with the inventory.
  CLI and Python expose the same policy
- [ ] Integration tests (macOS CI leg): mutate-then-journal-revalidate equals fresh scan
  by engine digest; UUID mismatch, event-ID regression, and forced `MustScanSubDirs`
  each degrade correctly
- [ ] Cross-platform packaging: target-conditional dependency under `watch`, with the G1
  fallback everywhere else, ubuntu CI leg running the fallback end-to-end with digest
  equality, wheel smoke exercising a warm open through Python on both OSes
- [ ] Performance loop: new `warm-revalidate-replay` job; the one-deep-edit acceptance
  scenario on the reference tree (quiet, one-file-touched at depth ≥ 10, and
  one-file-deleted rows, full sweep as the paired control) and a churn transition
  (measure replay, dirty-scope work, full-image costs, and total wall — H43/H38); ledger
  entries either way; public runtime replay stays unavailable until acceptance

## Testing Strategy

The gate is a pure function: every row in the table gets a direct unit test, no platform
APIs involved. Production replay will be integration-tested on macOS (`#[cfg]`-gated,
running in the existing macos-latest CI leg): each test mutates a real temp tree between
snapshot and reopen, then asserts the journal-scoped index equals a fresh scan’s by
engine digest — the same equality the parallel-walker tests pin.
Degradations are forced, not simulated: a wrong stored UUID, a stored event ID above
current, an undersized `max_changed_fraction`. The performance harness needs no changes
to verify correctness: its oracle already digests every trial’s index, so a journal-path
trial that skips a real change fails the run loudly.
Linux and Windows CI prove replay falls back, and no-default-build-feature tests prove
native discovery stays removable.
These production replay tests are planned, not the existing resident-watcher tests.

The committed probe must spawn distinct capture and replay processes, with no fdu
process or application-owned stream alive during mutation.
Compare the replay-scoped candidate against an independent full scan on a quiesced
fixture, and deliberately remove one needed scope to prove the oracle fails.
Exercise duplicate delivery, deep append, rename, deletion, hard-link aliases,
allocated-only changes, root replacement, and `.gitignore` edits.
A changed control file must trigger the existing descendant classification invalidation
even when no descendant event was emitted.

Test cursor publication with cancellation, failed saves, concurrent refreshes, and
crashes before and after commit.
Native platform tests cover actual event delivery; deterministic injected records cover
overflow, ambiguous flags, and fallback decisions.
Immediate cross-process evidence does not establish 24-hour retention, reboot behavior,
or a disk moved between computers.
Report those as separate pending acceptance cells.

## Rollout Plan

First land the exploration probe and its reproducible immediate results.
Then implement the private cursor/gate and replay transaction, followed by public opt-in
journal verification after the correctness and performance acceptance runs.
Use `watch` for packaging and retain full verification as the runtime default.
The README and skill text may only claim what the ledger shows, per the existing
no-unmeasured-claims convention.

## Open Questions

- Does `fseventsd` directory granularity hold under high-frequency mixed workloads (the
  research doc’s open question 3)? The churn experiment answers it empirically; G9
  bounds the damage if the answer is unfavorable.
- Cursor-per-volume for multi-volume scans: deferred behind G2 + `one_filesystem` now;
  the tag byte leaves room for a multi-cursor section later.
- Which completed per-device boundary can safely advance a quiet root’s cursor?
  The probe must establish this before production publication can be specified fully.
- What age and elapsed-time verification limits support measured daily use?
  The provisional 24-hour limit does not establish that use case.
- Is a fail-instead-of-fallback option useful beyond tests?
  Initially the public proposal allows the stronger sweep fallback and reports its
  reason.

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
  modern bindings, reconsidered as the preferred candidate for dependency review
- [Persistent-change prior art](../../research/research-2026-09-27-persistent-change-prior-art.md)
  — backup-product precedents, current Rust bindings, and platform limitations
- [Watchman fsevents resync](https://facebook.github.io/watchman/docs/troubleshooting.html)
- [End-to-end performance testing plan](plan-2026-08-09-fdu-end-to-end-performance-testing.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
