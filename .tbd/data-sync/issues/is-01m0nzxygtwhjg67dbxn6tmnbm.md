---
type: is
id: is-01m0nzxygtwhjg67dbxn6tmnbm
title: "PR #42 review R13: surface architecture doc says 126 golden sessions; the corpus has 129"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:05.593Z
updated_at: 2026-08-23T00:39:57.153Z
closed_at: 2026-08-23T00:39:57.153Z
close_reason: Fixed, with a correction to the finding. 126 was right in the parity spec -- it is the compared count, since three sessions are declined by name. The architecture doc's 126 was wrong because it describes what the corpus records, which is 129. The genuinely stale figure was 108/18, now 106/20, and that drift predates this branch (the deviations artifact is byte-identical to main's).
---
docs/project/architecture/fdu-surface-architecture.md:65. npm run test:golden reports '129 passed' across 9 files. The parity spec carries the same stale 126 in three places (pre-existing).
