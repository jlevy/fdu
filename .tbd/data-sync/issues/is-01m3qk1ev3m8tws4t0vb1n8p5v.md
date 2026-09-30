---
type: is
id: is-01m3qk1ev3m8tws4t0vb1n8p5v
title: "H187: the tree tier's consumer as one structural composite (sort only kept kinds, scalar roll-up for folded files, byte-keyed directory map)"
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
hold: null
hold_until: null
created_at: 2026-09-29T22:02:23.459Z
updated_at: 2026-09-30T01:55:07.033Z
started_at: 2026-09-29T23:08:09.166Z
closed_at: 2026-09-30T01:55:07.033Z
close_reason: "Rejected in exp-200: a 20% and 38% consumer instruction cut with identical answers reached no wall on either tree, because the tree route's consumer has slack on four vCPUs, unlike the summary consumer H188 cut. Reverted in 5df306ee; cfae174e stays reachable."
resolution: null
duplicate_of: null
---
60-75M consumer instructions on linux-v6.12: the name sort of every listing (28M) although the tier keeps 5.9k names, two empty by_ext maps per folded file (21M), and the PathBuf directory map hashed by component (19M). Predicted default-tree -3% to -5% (linux-v6.12) and -2% to -4% (node-modules-dense); secondary consumer instructions -35%; cold-scan-index non-inferior; answers identical including repeated names in a listing.

## Notes

Built cfae174e: ListingOrder (open-addressed table of listed names for the repeated-name dedup; kept kinds sorted; files folded as listed), InternedPartitionRollUp::add_file scalar path, directory map keyed by OsString. Consumer 228.5M -> 182.2M (linux-v6.12), 103.1M -> 64.2M (node-modules-dense); 129 tests, clippy, 212 goldens, 54 answer comparisons identical. Cell (exp-200) pending: control perf_probe-h186-a356d456, candidate perf_probe-h187-cfae174e.
