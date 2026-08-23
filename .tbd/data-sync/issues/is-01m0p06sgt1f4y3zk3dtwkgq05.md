---
type: is
id: is-01m0p06sgt1f4y3zk3dtwkgq05
title: "PR #42 R4: benchmarks README names the wrong package for perf_probe"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:55.385Z
updated_at: 2026-08-23T00:57:54.132Z
closed_at: 2026-08-23T00:57:54.132Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
benchmarks/README.md:79 says -p fdu; the example moved to fdu-core.
