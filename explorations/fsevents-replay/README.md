# FSEvents Replay Probe

A standalone macOS research instrument for `fdu-uwhl`. It tests persistent replay with
no probe process alive during fixture mutations.
It does not implement fdu refresh or establish journal completeness.

`probe.c` uses the installed Apple SDK and a serial dispatch queue.
`run.py` uses Python’s standard library to create isolated fixtures, launch separate
capture and replay processes, and compare a candidate inventory with a full `lstat`
walk. No dependency or build feature is added to fdu.

## Run

Requires macOS, Xcode Command Line Tools, and Python 3.12 or newer.
Choose a new directory on the volume under test; `prepare` refuses an existing one.
Store long-gap fixtures somewhere the operating system will not clean automatically.
Set `FDU_PROBE_SCRATCH` to the chosen scratch directory on the test volume and configure
`TMPDIR` and `UV_CACHE_DIR` there before invoking the commands.
Require the volume to be mounted; do not fall back to an internal-disk temporary path.

```shell
uv run --no-project python explorations/fsevents-replay/run.py self-test
uv run --no-project python explorations/fsevents-replay/run.py prepare "$FDU_PROBE_SCRATCH/example"
uv run --no-project python explorations/fsevents-replay/run.py replay "$FDU_PROBE_SCRATCH/example" --output "$FDU_PROBE_SCRATCH/example/quiet.json"
uv run --no-project python explorations/fsevents-replay/run.py mutate "$FDU_PROBE_SCRATCH/example"
uv run --no-project python explorations/fsevents-replay/run.py replay "$FDU_PROBE_SCRATCH/example"
```

Use a separate fixture and `mutate --scenario deep` to isolate a deep in-place append.
The default `mixed` scenario adds file replacement, deletion, directory rename, and a
new subtree. Mutations refuse a fixture that has changed since preparation.
The harness never purges journal history or removes its fixture directory.

For 1-hour, 24-hour, 48-hour, and 7-day gaps, prepare and mutate four separate fixtures,
then invoke `replay` on each at the intended elapsed time.
Both the cursor and mutation ages are recorded.
Exit the shell or reboot between phases to test longer process and system absence;
record the reboot separately because the harness does not detect one.
No resident process is needed between commands.
Retain unique evidence in the repository before retiring disposable fixture directories.

## Protocol and Records

Preparation drains the initial fixture events, stops and destroys that stream, saves its
latest event ID with the journal UUID, and independently scans the baseline.
This fence precedes all test mutations.
Preparation records hashes of both sources and the compiled helper, plus compiler and
SDK versions. Replay refuses a changed source or helper, so a long-gap run cannot report
new source provenance for an older executable.
The preliminary device-time fence and drain summary are also retained: using the
conservative time lookup directly can include fixture construction and widen scope.
The SDK specifies Unix-epoch seconds for `FSEventsGetLastEventIdForDeviceBeforeTime`,
despite its `CFAbsoluteTime` parameter type.

Each replay uses a device-relative stream and records exact event IDs and flags:

| Create flags | Meaning |
| --- | --- |
| `0` | Directory events |
| `128` | Directory events with `FullHistory` |
| `16` | `FileEvents` |
| `144` | `FileEvents` with `FullHistory` |

The four modes run in both orders, eight independent processes per invocation.
Replay waits for `HistoryDone`, calls `FSEventStreamFlushSync` for buffered contemporary
events, then stops, invalidates, drains the dispatch queue, and releases callback state.
A historical wait has a ten-second deadline; the parent kills any helper exceeding
twenty seconds, including a stuck flush or teardown.
The event trace is capped at 100,000 records and overflow fails the process.
Timeouts and helper failures fail loudly rather than becoming empty successful replays.

Results retain historical and total timings, sentinel IDs, the stream’s latest ID,
whether events arrived after `HistoryDone`, overlap counts, degradation reasons,
normalized scopes, and full-oracle mismatches.
Fixture-relative paths are hex encoded to retain arbitrary filename bytes without
leaking absolute paths.
Ancestor paths are classified without recording their names.
Journal UUIDs and device IDs remain in local state, outside committed observations.

Directory events rescan their named subtree; file events rescan their parent.
Ancestor events broaden to the fixture root.
Nested scopes are deduplicated, but events at or below the cursor are retained.
The candidate updates parent metadata and compares path membership, kind/mode, apparent
size, allocated blocks, modification time, inode, and link count against the oracle.
The oracle also compares status-change time and device identity.
Omitting all dirty scopes is an explicit negative control that must differ on a mutated
fixture. The quiet case legitimately has no negative-control mismatch.
The self-tests cover this oracle, deletion with trailing-slash directory events, scope
overlap, malformed paths, and degradation flags attached to control records.
They also exercise helper failure/timeout diagnostics and source/binary provenance
rejection. Failed native trials remain in the result’s structured `failures` list and
produce a nonzero exit status.

## Recorded Observations: 2026-09-26

[The machine-readable record](observations-2026-09-26.json) contains 48 native replay
trials on macOS 26.5.2, arm64: 24 on internal APFS and 24 on external APFS. Each regime
has eight quiet, eight mixed, and eight deep-append trials.
The earlier internal records use format v1 and retain their tested source hashes; their
helper source was unchanged between compilation and replay, but the binary hash was not
recorded at preparation.
The external records use v2 and verify preparation provenance.
These are separate bounded micro-fixture observations, not a paired storage performance
comparison. All candidates matched the full oracle and none reported degradation.
The mixed fixture changed twelve metadata records; the deep fixture changed one file and
no directory metadata.
Their omitted-scope negative controls detected those changes.

Without `FullHistory`, quiet replay returned only `HistoryDone` and deep replay returned
one dirty-parent event plus that sentinel.
With `FullHistory`, quiet replay returned nineteen overlapping creation events plus the
sentinel; deep replay returned nineteen coalesced events plus the sentinel, including
overlap and the new change.
Normalization then selected the entire fixture root.
Mixed `FileEvents` without `FullHistory` selected six smaller scopes; its `FullHistory`
counterpart selected the root.
This supports the changed-scope mechanism while showing how historical overlap can erase
its work savings. Discarding overlap to improve timing would invalidate the experiment.

Helper replay timings were 9.1–38.3 ms in the internal records (cursors approximately
0.3–29 seconds old) and 4.4–12.6 ms in the external records (under one second old).
These are single-host observations, not paired performance verdicts or an age/cost
model.
They exclude compilation, parent process work, inventory load, reconciliation, and
saving. No trial delivered events after `HistoryDone`, so this run does not establish
whether the flush is necessary or sufficient under queued/concurrent mutations.

The following remain untested: reboot continuity, unclean capture termination,
controlled journal loss, 1-hour/24-hour/48-hour/7-day gaps, high churn, non-APFS
volumes, root moves, mount changes, hard-link effects outside dirty scopes, symlinks and
permission errors, and fdu’s admission/accounting semantics.
The candidate deliberately rescans subtrees; it is not the engine’s intended relisting
algorithm. A full-root candidate can pass while providing no incremental savings.
Neither `HistoryDone`, UUID equality, nor these oracle matches proves universal journal
completeness. Keep `fdu-uwhl` open for the remaining acceptance matrix.

See the
[proposed integration design](../../docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
Apple’s
[persistent event guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html)
and the installed SDK’s `FSEvents.h` document the journal and callback contracts used
here.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
