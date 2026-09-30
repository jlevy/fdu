---
type: is
id: is-01m3qk1ecctt5q77ye65njtxe2
title: "H186: the default tree's serial tail (share threshold before rows, borrowed-name sort, detached release of the folded index)"
kind: task
status: in_progress
priority: 2
version: 4
spec_path: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
hold: null
hold_until: null
created_at: 2026-09-29T22:02:22.988Z
updated_at: 2026-09-29T23:15:50.651Z
started_at: 2026-09-29T23:08:08.541Z
---
The tail after the walk is 2.8 ms (linux-v6.12) and 4.5 ms (node-modules-dense): report_in builds and sorts a row per child before applying the share threshold, cloning two Strings per comparison; the folded index is freed inline below H156's threshold. Predicted -2.5% to -4% and -1.5% to -2.5% on default-tree. Also fixes the List view's machine-format sort (4.6G instructions on node-modules-dense). Placebo aggregate-summary --no-controls.

## Notes

Built a356d456: threshold before rows, RowFacts for omissions, borrowed-name sort (Cow) via borrowed_name helper, release threshold 64Ki -> 4Ki entries. Consumer 239.3M -> 228.5M (linux-v6.12), 120.6M -> 103.1M (node-modules-dense); 212 goldens pass; 171 extended answer comparisons identical. List-view JSON only 6.18G -> 5.76G: its sort costs more than the String clones (open item). Cell (exp-199) pending: control h188b, candidate perf_probe-h186-a356d456.
