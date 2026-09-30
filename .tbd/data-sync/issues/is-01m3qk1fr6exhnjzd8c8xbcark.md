---
type: is
id: is-01m3qk1fr6exhnjzd8c8xbcark
title: "H190: a second pass over Gitignore::decide (a per-source fast path for names that match no bucket)"
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
created_at: 2026-09-29T22:02:24.389Z
updated_at: 2026-09-30T01:55:07.402Z
---
About 400 instructions per governing source call, 111M on linux-v6.12; 99.2% of entries match no rule. Predicted -1.5% to -3% on default-tree linux-v6.12, likely short of the accept rule alone. Conditional on H187 landing and a fresh profile naming it at 3% of wall.

## Notes

Profiled on the H187 head 2026-09-29 and left unbuilt (no exact cut over 10M instructions). Behind exp-200's finding: a consumer-only cut on the tree route has no wall to buy on four vCPUs; reconsider only with a profile that puts the consumer on the critical path.
