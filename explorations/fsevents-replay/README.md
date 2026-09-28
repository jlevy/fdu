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

At this recording, the following remained untested: reboot continuity, unclean capture
termination, controlled journal loss, 1-hour/24-hour/48-hour/7-day gaps, high churn,
non-APFS volumes, root moves, mount changes, hard-link effects outside dirty scopes,
symlinks and permission errors, and fdu’s admission/accounting semantics.
The fixture candidate in `run.py` deliberately rescans subtrees; it is not the engine’s
intended relisting algorithm.
A full-root candidate can pass while providing no incremental savings.
Neither `HistoryDone`, UUID equality, nor these oracle matches proves universal journal
completeness. Keep `fdu-uwhl` open for the remaining acceptance matrix.

See the
[proposed integration design](../../docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
Apple’s
[persistent event guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html)
and the installed SDK’s `FSEvents.h` document the journal and callback contracts used
here.

## Read-Only Real-Folder Spike

`real_tree.py` extends the experiment to existing folders, without creating mutations
inside them. It inventories metadata only, never reads file contents, and never follows
symlinks. All state must be on an explicitly mounted external volume, disjoint from the
observed root. Directory traversal uses no-follow, descriptor-relative opens, including
every scope ancestor.
Cross-device entries and observation errors are reported rather than silently admitted
as complete coverage.

First prepare a small fixture with `run.py prepare` as above to build the native helper
and its provenance record.
Set `FDU_PROBE_ROOT` to the absolute folder to observe and `FDU_PROBE_STATE` to a new
external state directory whose parent exists.
For example, the root can be `$HOME/.claude` or `$HOME/.codex`; those are caller
choices, not built-in monitoring profiles.

```shell
uv run --no-project python explorations/fsevents-replay/real_tree.py self-test
uv run --no-project python explorations/fsevents-replay/real_tree.py baseline \
  "$FDU_PROBE_ROOT" "$FDU_PROBE_STATE" --alias sample \
  --helper "$FDU_PROBE_SCRATCH/example/probe"

# Exit; allow normal activity. No probe process needs to stay alive.
uv run --no-project python explorations/fsevents-replay/real_tree.py refresh \
  "$FDU_PROBE_ROOT" "$FDU_PROBE_STATE" --label later-01
```

The baseline saves a device-time fence before scanning, root identity, metadata, and
directory roll-ups. Refresh reuses the original fence with `FileEvents | FullHistory` by
default. Optional `--flags` values use the same matrix as the fixture probe; omitting
`FullHistory` is a diagnostic control, not a proposed safe optimization.
Each trial has a unique label.
The baseline and its cursor are never advanced or overwritten, so repeated trials test
increasing real gaps from the same before-state.
Changing the harness, native source, or helper invalidates its provenance: make a new
baseline instead of silently testing a different program against the old record.

The real-root candidate distinguishes shallow parent listings from recursive subtree
invalidation. Root-level file activity relists the root without walking every unchanged
descendant.
Separately nominated deep scopes remain necessary even when their ancestor is
relisted. New or replaced directories are discovered recursively; deleted subtrees are
removed after a successful containing-directory observation.
Directory lifecycle events and `MustScanSubDirs` request recursive work.
Ambiguous file-kind flags and global degradation select an explicit full-root fallback.
Observed hard-link changes expand to cached alias parents, without changing the
prototype’s per-path accounting.
A change through an unobserved external alias is still a history-coverage question, not
solved by that expansion.

One parent-to-children index avoids repeatedly walking the cached inventory for each
dirty directory. Its construction remains O(tree), is timed separately, and is not the
proposed persisted representation.
Summary fields distinguish a shallow root relist from recursive full-root fallback,
including fallbacks that happen to match the oracle.

Refresh independently scans an oracle before the candidate, constructs the candidate
only from the baseline and event-nominated observations, persists it, then scans a
second oracle. Stable mismatches fail with exit 1; differing oracle observations or scan
errors are inconclusive (exit 2), not a passing equality result.
Only an error-free equal comparison exits 0. Two matching observations are not an atomic
snapshot or universal completeness proof.
The negative-control self-test deliberately omits dirty scopes and must detect misses.

Timings distinguish preflight, load, replay, normalization, scoped observation,
roll-ups/diff, and candidate save.
The instrumented refresh segment excludes both oracles and reporting; it is not command
wall time or Rust fdu performance.
The before-oracle warms metadata caches and preflight warms the baseline file.
The full-scan control includes scanning, roll-ups, and saving, but is ordered after the
candidate, not a paired/interleaved performance trial.
Observed-entry counts and full-root scope widening are more useful mechanism evidence
than an unqualified speed ratio.

This prototype sums regular-file bytes per path, without hard-link or APFS-clone
deduplication. It does not reproduce fdu’s admission, content analysis, or complete
accounting contract.
It also still loads, copies, recomputes roll-ups over, and saves the flat inventory;
reduced filesystem work need not imply reduced whole-command cost.

Resource limits are explicit baseline arguments: `--max-entries` (default 1 million),
`--max-state-bytes` (2 GiB), and `--reserve-free-bytes` (2 GiB). Refresh inherits them.
Reaching a limit fails instead of falling back to internal storage.
State directories are private, and atomic files use private permissions.
Only `baseline-summary.json` and each trial’s `summary-shareable.json` are intended for
publication after inspection.
All other files contain private names or identities; hex encoding a path does not
anonymize it.

The
[prior-art update](../../docs/project/research/research-2026-09-27-persistent-change-prior-art.md)
distinguishes this experiment from shipping backup-product precedents and documents
Watchman’s warning that flush completion is not a current-state barrier under load.

## Real Roots and Next-Day Replay: 2026-09-27

[The sanitized record](observations-2026-09-27.json) retains both real-root summaries
and synthetic next-day failure traces.
Private filenames, journal UUIDs, and absolute root/state paths are omitted.
The host was running other builds; these observations are not an interleaved performance
verdict.

The real-root runs used harness SHA-256
`f620be13fc361fb29015fb760900c5c76c1d0621e2113961ca1d48aed1f85a95`. Parent review
subsequently fixed three strict-type-check findings: context-manager return annotations
and a summary-list cast.
The [exact annotation patch](harness-type-annotations.patch) preserves the recorded
source: in an isolated checkout at commit `494169b8`, reverse-apply it to `real_tree.py`
and verify that hash before resuming these old baselines.
The current script intentionally refuses their older source hash; do not edit the saved
provenance to bypass that check.
New captures use the corrected script.
These changes affect annotations and summary typing, not traversal, replay, or the
measured refresh segment.

| Real root | Baseline entries | Gap after baseline | Candidate observations | Outcome |
| --- | ---: | ---: | ---: | --- |
| Agent state A (`.claude`) | 12,280 | 79 s | 0 entries | Quiet; exact match against both full oracles |
| Agent state B (`.codex`) | 441,777 | 32 s | 441,796 entries | Full-root scope; zero stable mismatches, 38 concurrent paths; inconclusive |

The larger baseline contained about 37.9 GB apparent regular-file bytes.
Replay itself took 181 ms, but the conservative file-to-parent-to-subtree normalization
selected the entire root: scoped observation took 19.4 s, and the instrumented refresh
segment took 28.2 s. Its ordered full-scan/roll-up/save control took 23.1 s. These are
Python timings under live load, not evidence that the production engine would be faster
or slower by that ratio.
The candidate differed from its baseline in 63 metadata entries.
All 441,758 paths observed identically by both oracles matched the candidate; 38 paths
changed between oracles, including 19 for which the candidate matched neither endpoint.
Those may reflect intermediate states; they remain unverified, not explained away as
success. No observation errors occurred.

This result makes the planned distinction between immediate directory relisting and
recursive invalidation essential.
A file changing directly under the home or agent root must not automatically force
enumeration of all its unchanged descendants.
The next instrument should exercise that normalization with additions, deletions,
renames, unknown kinds, and degradation flags before any production integration.
Flat-image load, roll-up, and save costs also remain proportional to the whole tree.

The preserved external APFS fixtures enabled a genuine next-day test without waiting
another day: both known mutation sets were about 26 hours old.
All sixteen attempts (two scenarios, four create-flag modes, both orderings) received
events but failed the ten-second historical-completion deadline: no `HistoryDone`
arrived before timeout.
No trial was accepted or compared as a completed replay.
The delivered traces include the deep-append event, so this is not evidence that the
change was forgotten.
It is a failure to establish bounded completion; whether the cause is volume-history
traversal cost, completion semantics, or another condition requires a separate
instrumented test. Do not remove the deadline, ignore `HistoryDone`, or count these as
passing retention trials.
No reboot or journal-loss injection was performed.

**Disposition:** the basic mechanism has production precedent and local short-gap
evidence, but this prototype does not yet validate fast daily large-tree refresh.
Keep the real baselines for longer-gap trials; resolve scope inflation and replay
completion before the engine implementation gate.

## Shallow Refresh and Open Writers: 2026-09-27 Continuation

[The continuation record](observations-2026-09-27-continuation.json) retains the
controlled growth test, both real-root trials, native regression matrix, historical
completion diagnostics, and open-writer control.
These are mechanism tests on one live macOS/APFS host, not paired performance
measurements.

| Workload | Baseline entries | Candidate observations | Result |
| --- | ---: | ---: | --- |
| Controlled growth | 20,204 | 712 | Exact match against both full oracles; eight directory listings |
| Agent state A | 12,280 | 0 | Quiet; exact match |
| Agent state B, first refresh | 454,775 | 848 | One stable mismatch and 19 concurrent paths; failed |
| Agent state B, repeat from same fence | 454,775 | 900 | Two stable mismatches and 34 concurrent paths; failed |

The controlled workload became 20,206 entries and gained 66,516 apparent bytes and
61,440 allocated bytes.
It exercised root and nested edits, deletion, a directory rename, creation of a new
subtree, and an edit through a hard link.
The hard-link alias’s own parent received an event, so this workload did not exercise
the alias-expansion path; only the unit test does.
Every changed file also shared a directory with an evented entry, so the exact match
does not test per-file coverage.
Shallow root relisting did not traverse its unchanged descendants.
The native helper also passed sixteen fresh quiet/mixed regression trials across all
four flag modes and both orders.

The live root avoided recursive fallback, but correctness did not pass: the first stable
miss retained the baseline value for a file that had grown by 23,948 apparent bytes
after the saved fence.
Both full oracles agreed on its newer metadata, and its parent was absent from the
nominated scopes. The same file remained missed on repeat.
A targeted `lsof` query of that one file found a read/write descriptor.
No real folder was mutated to investigate this, and no private path appears in the
shared record.

The reported stable-miss counts understate the gap.
The oracle comparison labels any path whose two oracles differ “concurrent”, which mixes
changes made after the replay with changes made before it that were never nominated.
The sibling relist also refreshes unevented files that share a directory with an evented
one, so they count as stable-equal.
Re-classified against the baseline, **13 files (first refresh) and 14 (repeat) changed
before the replay with no event of their own**: the 1 and 2 reported stable misses, 4
and 4 stale baselines inside the concurrent set, and 8 and 8 files refreshed only
because a sibling had an event.
Every one was a session log or SQLite write-ahead log, and 12 of 13 and 13 of 14 were
held open read/write by long-running agent processes when checked.
Re-running the normalization on the saved traces reproduces the saved scope plans
exactly, so no nomination was dropped.
See the
[change-source review](../../docs/project/research/research-2026-09-27-disk-growth-change-sources.md#the-live-root-misses-were-open-writers)
for the audit.

The first live refresh spent 54 ms in replay and 1.77 s in scoped observation, including
1.47 s building the parent index.
Its measured refresh segment was 10.98 s because flat-image load, roll-ups/diff, and
save still dominate.
That is a *failed candidate’s* cost, not an accepted speedup.
The controlled candidate’s segment was 1.03 s. The before-oracle warms metadata caches;
these numbers retain the ordering and measurement limitations above.

### Reproducing controlled growth

Choose new, disjoint external paths for the owned fixture and its state.
The workload uses about 20,000 small files; it refuses reused preparation paths and
repeated mutations. Its mutation helper uses no-follow descriptor-relative access and
validates existing targets before writing.

```shell
uv run --no-project python explorations/fsevents-replay/fixture_workload.py prepare "$FDU_GROWTH_ROOT"
uv run --no-project python explorations/fsevents-replay/real_tree.py baseline \
  "$FDU_GROWTH_ROOT" "$FDU_GROWTH_STATE" --alias controlled-growth \
  --helper "$FDU_PROBE_SCRATCH/example/probe"
uv run --no-project python explorations/fsevents-replay/fixture_workload.py mutate "$FDU_GROWTH_ROOT"
uv run --no-project python explorations/fsevents-replay/real_tree.py refresh \
  "$FDU_GROWTH_ROOT" "$FDU_GROWTH_STATE" --label fullhistory-01
```

### Historical completion is a separate cost

`history_completion.py` deliberately permits a new replay instrument against preserved
synthetic capture state and records both provenances.
It validates the saved helper, checks replay sources around compilation and execution,
and records the new compiler and SDK. This differs from `run.py`’s exact-source-resume
protocol; no stored provenance or cursor is edited.
Partial JSON at a timeout is an explicit parse failure with private raw output retained,
never an accepted empty replay.

```shell
uv run --no-project python explorations/fsevents-replay/history_completion.py \
  "$FDU_PROBE_SCRATCH/aged-fixture" "$FDU_PROBE_SCRATCH/new-diagnostic" \
  --seconds 120 --mode wait --flags 144
```

The unchanged native default is ten seconds.
Diagnostic waits are explicitly bounded at up to 120 seconds, with a parent bound ten
seconds longer. Initial day-old replays completed at about 32–40 seconds; subsequent
final-instrument trials timed out at sixty seconds, and a 120-second diagnostic
completed at 92.5 seconds with exact oracle agreement.
In that last run, the first callback arrived in 8.84 ms.
Completion latency is therefore not explained by the number of dirty entries alone.

With an initial asynchronous flush, which itself returned in under a millisecond,
`HistoryDone` still did not arrive within ten seconds.
Synchronous flushing at startup required parent termination at twenty seconds; that
flush waits for the same historical processing, so it measures the same cost.
A no-`FullHistory` control also failed its parent bound, so removing overlap is neither
a validated fix nor a safe optimization.
A killed run without a final summary cannot localize delay to history, flush, or
teardown. Do not raise product deadlines to match these diagnostic tails: budget the
complete attempt against the full-scan alternative.

### An open writer can be invisible to completed replay

`open_writer.py` holds a synthetic regular-file descriptor open across baseline capture,
append, `fsync`, and a fresh replay subprocess.
It then closes the writer and replays the same cursor.
Each stage independently scans before and after the candidate.
There is no resident FSEvents probe during the mutation; the synthetic writer stays
alive as an application would.

A newly created fixture was inconclusive: overlapping creation events nominated the file
even though no fresh write nomination arrived while it was open.
An older owned fixture removed that masking.
With flags 144 and a genuine pre-append device-time fence, the while-open stage
completed with zero change events, zero overlap, and one stable mismatch.
After close, a fresh event arrived and the candidate matched both oracles.
This directly demonstrates a history-only refresh limitation for the tested workload; it
does not establish the exact cause of every live application mismatch.
The final instrument reproduced this on two untouched aged leaves, including an
independent parent-agent run.
A fresh-file control under the same final source remained overlap-masked.
`diagnostic_valid: true` means both experiment stages completed with matching identities
and no observation errors; it does not mean the while-open refresh was correct.
Invalid experiment stages exit nonzero.

```shell
# Use only an existing tree created by fixture_workload.py, with its ownership marker.
# Overlapping creation events make the result inconclusive; check the leaf's own record.
uv run --no-project python explorations/fsevents-replay/open_writer.py \
  "$FDU_PROBE_SCRATCH/new-open-writer-trial" \
  --helper "$FDU_PROBE_SCRATCH/example/probe" \
  --owned-fixture "$FDU_GROWTH_ROOT" --leaf branch-150/file-000
```

Apple’s published
[XNU close path](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/vfs/vfs_vnops.c)
contains a content-modified notification for written descriptors, consistent with this
test. It fires on the last close of the open file description, so a `dup`ed descriptor
delays it. Writes through a shared mapping are notified at the last unmap instead;
`fsync` never notifies.
A later live-stream matrix reproduced the same omission without replay, so a resident
watcher shares it; see the
[change-source review](../../docs/project/research/research-2026-09-27-disk-growth-change-sources.md#fsevents-cannot-see-open-writers-replayed-or-live).

Creation-history overlap depends on journal segmentation, not wall-clock age: on the
churning external volume, replay re-delivered creation records from ten minutes before
the fence. The overlap-independent criterion is a new record, or a new `Modified` bit,
for the leaf itself.

**Disposition:** keep the spike and its negative controls; do not enable production
history-only refresh.
Shallow normalization works on the controlled workload, but active-writer coverage and
historical-completion cost remain blockers for the intended daily workflow.
Track active-writer coverage in `fdu-vhrb`, the longer acceptance matrix in `fdu-uwhl`,
and the continuation evidence in `fdu-uz5r`. Generic native writer discovery is a next
experiment, not an implemented solution; permissions or unavailable coverage must lead
to an explicit limitation or scan fallback.
Suggested roots remain skill/profile policy, never a list embedded in the engine.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
