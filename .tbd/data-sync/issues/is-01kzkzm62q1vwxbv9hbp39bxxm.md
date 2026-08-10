---
type: is
id: is-01kzkzm62q1vwxbv9hbp39bxxm
title: Build reproducible end-to-end performance evidence for fdu
kind: epic
status: open
priority: 1
version: 21
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
  - is-01kzn04cg15xabhzvnnnmraqs3
  - is-01kzn04cqdaaknww7941cbp7aw
  - is-01kzn13r7sqzmfvrsjx317p7dt
  - is-01kzn2jfw8mvmrmarg3fxksxgb
  - is-01kzn35a06843ecqt0q47w1j9w
  - is-01kzn3wqxg32et4d1zksqf6hf9
  - is-01kzn3wr63m47hmqh90ep0aczz
  - is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-09T19:22:34.966Z
updated_at: 2026-08-10T22:13:19.646Z
---
Child evidence epic under fdu-qfz6. After the current merge blockers take priority, build deterministic corpora/oracle, then the strict state-machine runner, fdu probes and collectors, reviewed dut/gdu adapters, stable regression governance, and the final generated report. The harness supplies common evidence to Phase 1 owner beads, resets every trial state, rejects samples whose oracle fails, and introduces no performance claim until the optimized engine passes the dedicated-host matrix.

## Notes

Portable foundation is complete: corpus/oracle fdu-rq5m, strict runner fdu-d8kq, and probe/portable collectors fdu-oj25 are closed. Exact probe validation also closed correctness bead fdu-6x07 and portable metadata optimization fdu-s23t. Claim-grade provenance and intrusive Linux diagnostics are split into fdu-849g and fdu-bmhr, which block stable/release evidence but not the now-unblocked revalidation and snapshot spikes. No current timing supports a product claim.
