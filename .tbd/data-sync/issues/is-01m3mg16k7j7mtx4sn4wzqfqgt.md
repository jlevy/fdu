---
type: is
id: is-01m3mg16k7j7mtx4sn4wzqfqgt
title: "Decide: --stale-ok answers are barely marked stale in plain text (-q removes the only marker)"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:12:05.989Z
updated_at: 2026-09-28T23:24:37.058Z
closed_at: 2026-09-28T23:24:37.057Z
close_reason: "Decision: plain text carries a visible marker -q keeps (maintainer: no preference; orchestrator chose per the design principle). eb97d7f7: warn: stale answer line on stderr for every format when provenance.source is cache_only; Python Report.warnings added (additive); goldens and parity updated. Merged on PR #155."
resolution: null
duplicate_of: null
---
With --stale-ok, plain text says only 'cache only' in the perf footer, and -q removes it; docs/usage.md says the answer is 'labelled stale'. Not a regression. Decide whether plain text should carry a visible stale marker that -q keeps, or fix the docs.
