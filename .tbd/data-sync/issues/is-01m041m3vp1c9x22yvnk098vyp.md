---
type: is
id: is-01m041m3vp1c9x22yvnk098vyp
title: Adaptive worker threshold sits inside the operating range of real macOS trees
kind: bug
status: open
priority: 2
version: 2
labels:
  - performance
  - macos
  - campaign-2
dependencies: []
created_at: 2026-08-16T01:05:20.757Z
updated_at: 2026-08-23T09:09:04.425Z
---
ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY is 30,000 ns. On /Users/levy/wrk/aisw/trading
(505,415 entries, 129,833 dirs, ~2.9 files/dir, macOS/APFS) the calibration window
measures 20.6-41.0 us/entry depending on machine load -- straddling the threshold.

Observed on one binary, one tree, same session:
  --view summary run A: 20.6 us/entry -> reserve expansions 0 (stayed at 6 workers)
  --view summary run B: 32.9 us/entry -> reserve expansions 1 (scaled to 16)
  default path, 12 runs: 31.8-41.0 us/entry -> scaled every time

So the worker count for a given tree is decided by whichever side of the threshold the
first 16k entries happen to land on, which moves with host load. 6 vs 16 workers is a
large behavioural difference: it shows up directly in aggregate kernel time (a user
reported sys 28.5 s where a lighter-loaded run of the same command measured 19.6 s).

This is the documented consequence of a one-shot calibration (scan::tests::
completion_order pins that one tree can decide either way), but platform-tuning.md
flags this constant as the clearest suspected mismatch on the grounds that it is
suspected INERT on Linux (1.5 us/entry, twenty times below the trigger). This is the
opposite failure on macOS: not inert, but marginal, and therefore nondeterministic.

Consequences: (1) benchmark noise -- paired fdu-vs-dust runs on this tree return a
statistical tie with a 95% CI of [-508, +66] ms because per-pair swing reaches +/-800
ms; (2) users see run-to-run variance they reasonably read as a regression.

Not a correctness issue and not obviously a wall-time loss -- a thread sweep on this
tree (perf_probe scan-index, 4/6/8/10/12/16) found <=4% spread, inside noise, so
neither side of the decision is clearly better here. That is itself the argument:
if the decision does not matter for wall time, it should not be a coin flip that
moves aggregate kernel time by 45%.

Options to weigh: hysteresis or a second calibration window before committing;
widening the window; deciding on a percentile rather than a mean; or -- if the sweep
result generalises -- questioning whether the reserve expansion earns its complexity
on macOS at all. Needs a quiet host (fdu-ow8y) before any of it is measurable.
