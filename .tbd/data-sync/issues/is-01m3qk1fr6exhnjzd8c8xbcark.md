---
type: is
id: is-01m3qk1fr6exhnjzd8c8xbcark
title: "H190: a second pass over Gitignore::decide (a per-source fast path for names that match no bucket)"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
created_at: 2026-09-29T22:02:24.389Z
updated_at: 2026-09-29T23:21:25.061Z
---
About 400 instructions per governing source call, 111M on linux-v6.12; 99.2% of entries match no rule. Predicted -1.5% to -3% on default-tree linux-v6.12, likely short of the accept rule alone. Conditional on H187 landing and a fresh profile naming it at 3% of wall.

## Notes

Profiled on the H187 head cfae174e (callgrind, line-level): .gitignore matching ~100M of the 182M consumer instructions on linux-v6.12; decide 545 instr per source call: Name::new 8M, probes 10M, residual pre-checks 25M (~9 candidates/call), holds_literals 14M (96k calls), tails loop 5M. No exact cut above 10M; a per-source name memo would need stored names for exactness for a predicted -2% on one subject. Left unbuilt; conditional stays.
