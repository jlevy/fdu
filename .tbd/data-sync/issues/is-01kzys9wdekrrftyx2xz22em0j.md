---
type: is
id: is-01kzys9wdekrrftyx2xz22em0j
title: Probe job for the transient summary tier, with a tallies oracle
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T00:03:44.686Z
updated_at: 2026-08-14T00:03:44.686Z
---
The aggregate tier (RetainedState::Summary) has no perf_probe mode and no measure.py job, so it cannot be A/B measured under the accept rule and has no component_ns. It is reachable only through compare_tools.py driving the real CLI, so every layer-1 number carries process spawn, arg parsing, canonicalize and JSON rendering. exp-043 and exp-044 both resolved on wall changes of +0.67% and -1.15% while user CPU fell 40% and 50%, with no component timer available to tell dilution from truth. The blocker is structural: summarize_index builds the verification digest by walking the index, and this tier retains no index. Needs a tallies-based oracle (files, dirs, apparent bytes, allocated bytes, newest mtime) checked against the tree fingerprint, as compare_tools already does.
