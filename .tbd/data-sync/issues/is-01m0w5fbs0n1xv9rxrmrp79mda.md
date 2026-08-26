---
type: is
id: is-01m0w5fbs0n1xv9rxrmrp79mda
title: Exclude special filesystem objects at the MetaBrowser provider boundary
kind: bug
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47
    at: 2026-08-25T09:54:29.343Z
  - kind: pr
    url: https://github.com/jlevy/metabrowser/pull/74
    at: 2026-08-25T09:54:29.344Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:38.318Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5018663775
    at: 2026-08-25T12:07:06.276Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T09:54:25.695Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-25T11:28:11.417Z
close_reason: null
resolution: null
duplicate_of: null
---
Exact-head adoption finding at FDU d19b0ce versus MetaBrowser 0577bb1. MetaBrowser exposes a closed EntryType algebra of file, directory, and symlink and requires every provider to exclude sockets, FIFOs, devices, and other special objects rather than reclassify them (arch-inventory-provider.md:188-193; contract.py:366-369). Its Python walker excludes them before retention and refresh removes them. FDU retains EntryKind::Other in the authoritative index (scan.rs:4309-4319), and the reference embedder listing maps every non-directory row through the same file-shaped Row path without checking kind (browser_provider.py:204-219). A thin adapter cannot repair native rollups, continuation remainders, change invalidations, and max_files semantics after the fact if Other remains in projections as an ordinary leaf. Define the engine-native MetaBrowser projection/admission rule: special objects may remain valid for FDU CLI semantics, but the opened provider view must exclude them consistently from listing/tree/catalog pages, rollups/remainders, refresh/watch output, diagnostics, and agreement fixtures without a Python mirror or second filter index. Acceptance: boot plus create/delete/replace scenarios for FIFO/socket where supported; three-kind output only; exact conservation and paging; no special object counted as a regular file; both providers agree from one recorded observation stream.

## Notes

Shipped as an engine-native scope axis: ScanConfig::exclude_special and
ScanScope::exclude_special, surfaced as `--special keep|prune` and
ScanOptions.special. No Python mirror and no second filter index -- the entry
is not in the authoritative index at all, so listing, tree, catalog pages,
rollups, remainders, refresh and watch output all describe one inventory
without any of them filtering.

One predicate, scan::retains, asked wherever a kind first becomes known --
after the metadata read, since a name does not say whether it belongs to a
socket. Seven sites: the serial walk, the parallel walk, both reconcilers, the
single-path refresh, and watch::retained, which is the watcher's apply funnel
and the third producer of rows.

Three decisions worth recording:

- The guard went into revalidate first, whose listing loop looks exactly like
  the reconcilers' and is not on their path. A scan excluded the socket and the
  first refresh put it back. Found by writing the tests before believing the
  wiring.
- Excluding is removing, not skipping. Every listing loop takes the name out of
  its missing-set before the kind is known, so a `continue` leaves the old row
  standing over a socket forever. Each site emits Op::Remove.
- The watcher's substitution carries the producer's expectation across.
  Rebatching through Observation::new would flatten every arbitration
  precondition to Any.

Admission runs before the fdu-97dd budget claims a slot, so an out-of-scope
entry cannot spend one a retained entry could have used. No snapshot format
bump: the scope flags byte had a spare bit, and a clear bit means "kept", which
is what every earlier snapshot meant.

Acceptance:

- Boot: crates/fdu-core/tests/special_objects.rs, 12 tests -- default keeps,
  pruning gives three kinds only, exact conservation (files and dirs identical,
  `others` down by exactly one), the axis in ScanScope, and both walkers and
  both reconcilers checked separately.
- Create/delete/replace: create-after-scan, replace-in-place through the
  directory sweep, and replace-in-place through a single-path refresh, each
  under both scopes so a filter that ignores the flag fails.
- Watch: three tests in watch_session_integration.rs (create under a pruning
  watch, replace under one, and record under the default), plus three unit
  tests on watch::retained.
- Both providers from one observation stream: the reference embedder now opens
  with special="prune", folds the axis into its scope digest, and its fixture
  carries a live socket -- public_smoke.py asserts no fourth kind reaches a row
  at boot or across a refresh, and that the roll-up counts exactly the rows the
  listing shows.

Every guard was mutation-checked: reverted in turn, each with a named test that
fails. Two mutations survived the first pass and both were real gaps -- the
serial walker is unreachable on a multi-CPU host without threads=Some(1), and
drop-versus-remove is masked in a live session because a rename onto a watched
path escalates to a root invalidation on Linux and reconciliation sweeps the
stale row away. The first is now covered by a single-threaded test; the second
by a unit test, since no integration test on this backend can isolate it.

make check and make cross-lint pass; parity holds at 23 recorded deviations,
none added.
