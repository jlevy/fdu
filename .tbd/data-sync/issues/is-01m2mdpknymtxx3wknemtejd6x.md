---
type: is
id: is-01m2mdpknymtxx3wknemtejd6x
title: "PR #65 delta review PR65D-PERF-1: the published speed gate describes a head without #63's engine commits"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-16T06:15:39.965Z
updated_at: 2026-09-16T07:15:44.280Z
closed_at: 2026-09-16T07:15:44.279Z
close_reason: "Measured at 9311462 against the merge-base f047dab, 15 interleaved pairs per subject: control-free default tree 0.988 (0.980-1.035), summary 1.002 (0.990-1.023); control-rich default tree 0.999 (0.965-1.011), summary 1.023 (0.994-1.043). Worst ratio 1.023, inside the 1.10 gate; totals identical on both subjects. Covers both 581557c and b989aa9. The PR body's speed-gate section is replaced with the measurement and names the head."
resolution: null
duplicate_of: null
---
PR #65 delta review 5219107144. The gate table compares main against d95d729, whose merge-base with 9105768 is 1fd71a9, so none of #63's six engine commits were in the gated binary, and #63 published no gate. 581557c adds a file_name() compare and two BTreeMap::range probes per Op::Upsert on every scanner batch regardless of read_controls. Measure one interleaved 15-pair run at the merged head against origin/main on the control-free subject, and restate the PR body's speed-gate section as measured.
