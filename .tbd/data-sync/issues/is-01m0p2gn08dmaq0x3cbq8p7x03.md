---
type: is
id: is-01m0p2gn08dmaq0x3cbq8p7x03
title: Re-verify the metadata-walk physics evidence against the post-split crate layout
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/reports/report-2026-08-23-metadata-walk-floor.md
assignee: claude-code@vm
labels: []
dependencies: []
created_at: 2026-08-23T01:07:15.592Z
updated_at: 2026-08-23T01:16:05.702Z
closed_at: 2026-08-23T01:16:05.702Z
close_reason: null
---
origin/main split the workspace: crates/fdu-core is the engine and crates/fdu is the CLI
package on its public API. Every figure in the metadata-walk physics research, the
published artifact, and the README note added with it was measured on the pre-split
binary, and the two new spikes (parfloor.c, peerwalk.rs) were written against the old
layout.

The split moved the CLI onto the library's public API, which is exactly the layer the
aggregate tier's one-shot report plan runs through, so the measured numbers cannot be
assumed to carry.

Check, and repoint or re-measure whatever moved:
- build commands the docs and spikes README name
- source references (plan_report and the execution planner, the counters subsystem)
- the perf_probe example's package
- the paired fdu-vs-floor and fdu-vs-ignore figures, on a rebuilt binary

## Notes

Done. Post-split verification, 2026-08-23:

- plan_report and the execution planner: crates/fdu-core/src/execution.rs. The report
  names it without a path, so nothing to repoint.
- perf_probe: already repointed upstream to -p fdu-core in the Makefile.
- the fdu binary: still [[bin]] fdu in crates/fdu; cargo build --release --bin fdu works
  unchanged.
- neither new spike (parfloor.c, peerwalk.rs) nor the report referenced a crate path.

Re-measured on a binary rebuilt from the merged tree, paired and interleaved, 13 trials
after 3 warmups:

  tree 420k synthetic   fdu 197.6 ms  floor 164.3 ms  1.20x   vs ignore -21.5%
  usrnolnk real names   fdu  57.6 ms  floor  40.6 ms  1.42x   vs ignore  +1.7%
  /usr real tree        fdu  70.8 ms  floor  44.7 ms  1.59x   vs ignore +12.4%

against 1.17x / +1.5% / +11.8% before the split, with every tally identical. The split
did not move the aggregate tier. The report states which sweeps are pre-split.

This did surface a second corpus-flattered claim: the x-floor headline was the primary
synthetic subject quoted as if it were the program. Corrected to 1.20x synthetic against
1.59x real, with the per-subject table in the report.
