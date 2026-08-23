---
type: is
id: is-01m0nzxvb7zsac07x6bbdyzgwn
title: "PR #42 review R5: AGENTS.md and the instrumentation playbook link to crates/fdu/src/counters.rs"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:02.343Z
updated_at: 2026-08-23T00:39:54.570Z
closed_at: 2026-08-23T00:39:54.570Z
close_reason: Fixed. AGENTS.md and the instrumentation playbook link crates/fdu-core/src/counters.rs and name fdu_core::counters.
---
AGENTS.md:160 and docs/project/guides/performance-instrumentation-playbook.md:21 link a path that no longer exists, and name fdu::counters where it is now fdu_core::counters.
