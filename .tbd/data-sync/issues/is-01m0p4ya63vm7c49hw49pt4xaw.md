---
type: is
id: is-01m0p4ya63vm7c49hw49pt4xaw
title: "make perf-floor: the tier-by-subject floor scoreboard"
kind: task
status: in_progress
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - campaign-2
dependencies: []
child_order_hints:
  - is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-08-23T01:49:40.418Z
updated_at: 2026-09-13T21:40:20.046Z
---
Campaign 2 orders work by each tier's measured distance to the parallel syscall floor,
which makes that distance the scoreboard -- and today deriving it is a by-hand session
with the spikes. Add a harness entry point (make perf-floor) that:

- builds parfloor and peerwalk (benchmarks/spikes/) and the fdu binary
- runs the floor variants and the fdu tiers (aggregate, index, cache-only) across the
  nominated real-tree subject set plus the standard generated subject, paired and
  interleaved, with the shared tally oracle enforced
- emits the x-floor table per tier per subject, in a committed or easily diffable form

Every accepted change re-runs it, which is what makes the shared-cost re-screen and the
termination criteria in the campaign-2 plan checkable rather than asserted. The floor
report (docs/project/reports/report-2026-08-23-metadata-walk-floor.md) documents the
instruments and the protocol this automates.

## Notes

Blocked on a design decision, not on effort. parfloor.c -- the denominator every x-floor threshold in campaign 2 is defined against -- is Linux-only (SYS_getdents64, statx; no Darwin equivalents). arena_spike.rs and peerwalk.rs are portable. So a macOS scoreboard needs either a getattrlistbulk port of the floor or a different floor set with the regime difference recorded, and that should be decided in the plan rather than by a harness falling back. The Linux half is straightforwardly scriptable today.

2026-08-28 (Linux session): THE LINUX HALF IS LANDED. Branch claude/perf-floor-linux-2026-08-28.

`make perf-floor SUBJECTS="label=/path"` builds parfloor and arena_spike, runs them interleaved with the aggregate and index tiers under the shared tally oracle, and emits the x-floor table with each tier marked against its termination threshold. It times each instrument's own internal timer rather than the spawn wall. On a non-Linux host it refuses, naming the fdu-9hdc decision, rather than substituting a denominator and printing the same column heading.

First numbers, 4 vCPU x86_64 Linux container, 30 trials, 3 warmups, interleaved, quiet regime, commit b75bf85:

  /usr  (75,976 entries): parfloor-enum 0.46x | parfloor-stat 1.00x | aggregate 1.58x | index 2.82x | arena_spike 3.92x (bimodal)
  /opt  (47,819 entries): parfloor-enum 0.58x | parfloor-stat 1.00x | arena_spike 1.09x | aggregate 1.40x | index 2.93x

Three findings.

1. The aggregate ratio reproduces the by-hand floor report exactly: 1.58x here against the 1.59x recorded on /usr, on different hardware through an independently written harness. That is the first independent reproduction of any x-floor number in this campaign. The index tier confirms at 2.82-2.93x against its 1.40x threshold, which is H86's case restated on a third host.

2. arena_spike is BIMODAL on the 76k subject: a ~63 ms mode and a ~150 ms mode, selected by how much memory the preceding process churned. It is unimodal and 1.09x on the 48k subject, which reproduces the recorded 1.06x ceiling. This matters for H86 (fdu-xde5), whose pre-registered targets include peak RSS <=3x arena_spike and a tail-spread bound: if the ceiling has two modes on a subject of this size, both targets have to name which mode they mean. Note that p95/median -- the tail statistic the loop already records -- reads a reassuring 1.16 on that same distribution, because both humps are individually narrow. max/min reads 4.25. The harness therefore reports spread and flags anything at or past 2x.

3. The index tier's spawn wall exceeds its own component timer by 149 ms against a 110 ms component, so 57% of a spawn-timed index number is the probe's engine-digest oracle rather than the engine. The loop guide's 31.9% figure understates it for this tier. This sharpens fdu-4xtm (--no-oracle mode) from a tidiness item into a correctness one for any index-tier number timed around a process.

Not done, and unchanged: the macOS half (fdu-9hdc), still a decision rather than effort.

Caveat on the subjects: neither /usr nor /opt is nominated. This host is ephemeral and the nominations file is per-host and gitignored, so the x-floor ratios transfer but the absolute milliseconds do not. /usr clears the deciding bar on size (75,976 >= 50,000); /opt does not (47,819) and screens.

2026-09-13 (review of PR #49): formal review 5192251516 (https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516) at head 1fa2309 found 13 defects in the harness, none of which produce a silently wrong tally but several of which make the published ratios and labels less trustworthy than the table implies: the probe tiers ran fdu's automatic pool rather than the floor's thread count (FLOOR-1, High), the instrument order was a fixed rotation (FLOOR-2), quiet was checked once at entry rather than per trial (FLOOR-3), a spread-flagged tier could still read as closed (FLOOR-4), the probe build duplicated perf-probe-release (FLOOR-5), and 'entries' meant dirs+files here but root+dirs+files+symlinks in subjects.py (FLOOR-6), plus seven Low. Tracked under fdu-pfo7, one child bead per finding.

Two corrections to the notes above follow from it. /usr is 84,536 entries as perf-subjects counts them (75,976 directories and files); /opt's count under that definition was not recorded, so the '/opt screens (47,819)' classification does not stand. And arena_spike always ran immediately after parfloor-enum in the recorded run, so 'selected by how much memory the preceding process churned' is a hypothesis that schedule could not test.
