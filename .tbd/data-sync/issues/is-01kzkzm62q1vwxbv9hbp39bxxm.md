---
type: is
id: is-01kzkzm62q1vwxbv9hbp39bxxm
title: Build reproducible end-to-end performance evidence for fdu
kind: epic
status: open
priority: 1
version: 12
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzkzmrjbr2ew8wt774r1n26x
  - is-01kzkzmrsbmqxgen0n8sh13hfz
  - is-01kzkzms089qshzydhxvjfkdt5
  - is-01kzkzms7gmpjb0smwfc0c74wr
  - is-01kzkzmsegmx4sfswka2084se6
  - is-01kzg4c6h9v2dzand7t090p278
  - is-01kzkzmrbcwvtrfpgbpbs4vpw0
  - is-01kzmyvzzhag70nv3fh7rfhec7
created_at: 2026-08-09T19:22:34.966Z
updated_at: 2026-08-10T04:28:36.720Z
---
Child evidence epic under fdu-qfz6. After the current merge blockers take priority, build deterministic corpora/oracle, then the strict state-machine runner, fdu probes and collectors, reviewed dut/gdu adapters, stable regression governance, and the final generated report. The harness supplies common evidence to Phase 1 owner beads, resets every trial state, rejects samples whose oracle fails, and introduces no performance claim until the optimized engine passes the dedicated-host matrix.

## Notes

The self-contained design and six-bead implementation graph remain current. Supply-chain, correctness, and concurrency implementation prerequisites are closed; execution begins with fdu-rq5m after final PR approval fdu-sn43. No timing result yet supports a product claim.
