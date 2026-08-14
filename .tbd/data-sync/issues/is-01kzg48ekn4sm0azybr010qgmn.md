---
type: is
id: is-01kzg48ekn4sm0azybr010qgmn
title: "fdu phase 1: fastest walker with full stats, proven by benchmark"
kind: epic
status: open
priority: 1
version: 32
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
child_order_hints:
  - is-01kzky6vqxwd47xz3we21s86zq
  - is-01kzm3t12dcq5h7n92xztnhcyd
  - is-01kzm3nms9b78zn0nqqyy9sq26
  - is-01kzkzm62q1vwxbv9hbp39bxxm
  - is-01kzg4c75tvbrg6rgh3803nwzj
  - is-01kzg48zxv9jrjbrfswztx2q36
  - is-01kzg4908hkkpt0pf20602rze8
  - is-01kzg4akhzmh7xgcabnnyc4e9f
  - is-01kzg48z8ykg6t1de81nbvdqpw
  - is-01kzg48zktc7ager8tcy3cst7r
  - is-01kzg49rw1p40pjc18feb9ghpv
  - is-01kzg49s5s1gst3526wx73q9rf
  - is-01kzg49sfhtxshw3senkhjmc24
  - is-01kzg49sswr78gpjykxctbe6c7
  - is-01kzg4ajxc0pvgcmj834gahcgt
  - is-01kzg4ak7v8z2a7s41rsms8jcb
  - is-01kzg4akvjfp8s9h0a1vs7h1c4
  - is-01kzg4bey8nn4k8y1daxc9exhd
  - is-01kzg4bf862ajh8g2tmv5bznng
  - is-01kzg4bfj2cqzcksgpmfce89w6
  - is-01kzg4bfw0zmmztg25v9a0nkq4
  - is-01kzg4c6vnh98mqrpkzw7ydne0
  - is-01kzkskszrb20xkk7g3gt32za6
  - is-01kzmnx3taexx4cq4m722p0yp0
  - is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-08T07:26:35.637Z
updated_at: 2026-08-14T21:19:05.636Z
---
Root execution epic for fdu. PR #1 merged the Phase 0 product slice and all Wave 0 supply-chain, atomicity, ownership, concurrency, cross-platform, and final-validation gates into main. Execute the remaining Rust refactor guards and shared performance-evidence foundation, resolve measured design gates, build the optimized engine, finish product surfaces, publish the complete performance report, then release. The CLI UX and zero-install skill follow-up is owned by fdu-6c8n on a new branch from main. Future-only extensions live under fdu-x746 and do not compete with the critical path. Exit criteria include measured cold/full-stat, warm 500k, memory, CLI/schema, reproducible evidence, ratified goals, and bounded provable concurrency contracts from the linked plan.

## Notes

PR #1 merged at origin/main commit 92ee5ab. The next active P1 slice is the focused CLI UX/skill/wheel follow-up fdu-6c8n; no performance claim is made.
