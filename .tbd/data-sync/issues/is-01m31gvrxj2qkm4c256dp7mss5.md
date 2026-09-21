---
type: is
id: is-01m31gvrxj2qkm4c256dp7mss5
title: After leftover apply-timer expansion, re-profile Linux content-cache-hit restore mix (H149)
kind: task
status: in_progress
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzysa79temyc45zjn2v98kpw
created_at: 2026-09-21T08:21:02.513Z
updated_at: 2026-09-21T08:21:17.054Z
---
H144 (exp-144) named the Linux cache-hit leftover under the post-H112 apply bucket: apply ~80–84 ms, parse and candidates ~27 ms, read ~9 ms; no new ≥3% userspace cut.

fdu-2pct moved apply start to candidates.remove so HashMap remove + fingerprint sit in apply. H121/H144 mixes are not comparable to the new rows. Re-profile before another apply/install cut.

Determination (Linux, reconstructible linux-v6.12, quiet first):
- Phase split from FDU_COUNTERS=1 content-cache-hit (content_sidecar_{read,parse,candidates,apply}_us).
- A named restore stage is or is not ≥50% of restore and ≥3% of claim-grade wall.
- Same leftover identity as H144, or a new cut Darwin/H144 did not name.

Do not retry H116, H125, H129, H131, H133. Do not compile a cut unless the mix names one. Do not ship PORTABLE constants. Mint H149/exp-155 when the cell starts. Stack on #97; cherry-pick leftover timer 4d78558e so the binary matches the definition.
