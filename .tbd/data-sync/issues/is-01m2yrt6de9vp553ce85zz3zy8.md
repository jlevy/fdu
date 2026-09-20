---
type: is
id: is-01m2yrt6de9vp553ce85zz3zy8
title: "H85 screen: transient scan() still cross-thread frees on Linux"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies: []
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T06:42:16.110Z
updated_at: 2026-09-20T06:54:07.706Z
closed_at: 2026-09-20T06:54:07.706Z
close_reason: "H85 rejected against 20% bar (exp-150): quiet linux-v6.12 -4.98%; 450k screening -11.31% n=7; RSS flat. Do not lower H85. 3% keep is H147/exp-151."
---
H86 detached arenas did not consume RetainedState::Summary. prepare_report --no-controls still uses public scan() Observation batches: workers allocate PathBufs, the consumer frees them. Screen H85 (return drained batches to the producing worker) against the 20% mimalloc bar on transient aggregate. Do not ship PORTABLE. Do not restart H86. fdu-h7sw stays the standing bead.
