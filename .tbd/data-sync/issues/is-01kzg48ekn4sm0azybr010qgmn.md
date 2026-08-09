---
type: is
id: is-01kzg48ekn4sm0azybr010qgmn
title: "fdu phase 1: fastest walker with full stats, proven by benchmark"
kind: epic
status: open
priority: 1
version: 22
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
child_order_hints:
  - is-01kzg48z8ykg6t1de81nbvdqpw
  - is-01kzg48zktc7ager8tcy3cst7r
  - is-01kzg48zxv9jrjbrfswztx2q36
  - is-01kzg4908hkkpt0pf20602rze8
  - is-01kzg49rw1p40pjc18feb9ghpv
  - is-01kzg49s5s1gst3526wx73q9rf
  - is-01kzg49sfhtxshw3senkhjmc24
  - is-01kzg49sswr78gpjykxctbe6c7
  - is-01kzg4ajxc0pvgcmj834gahcgt
  - is-01kzg4ak7v8z2a7s41rsms8jcb
  - is-01kzg4akhzmh7xgcabnnyc4e9f
  - is-01kzg4akvjfp8s9h0a1vs7h1c4
  - is-01kzg4bey8nn4k8y1daxc9exhd
  - is-01kzg4bf862ajh8g2tmv5bznng
  - is-01kzg4bfj2cqzcksgpmfce89w6
  - is-01kzg4bfw0zmmztg25v9a0nkq4
  - is-01kzg4c6h9v2dzand7t090p278
  - is-01kzg4c6vnh98mqrpkzw7ydne0
  - is-01kzg4c75tvbrg6rgh3803nwzj
  - is-01kzkskszrb20xkk7g3gt32za6
  - is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-08T07:26:35.637Z
updated_at: 2026-08-09T19:22:34.966Z
---
Umbrella for phase 1. Phase 0 (scaffold) is done: delta contract, index with hierarchical reducers, portable walk+revalidate, snapshot lifecycle, notify-backed watch layer, CLI, Python bindings, CI. Phase 1 makes Goal 1 true and demonstrable.

Exit criteria (all must hold):
1. Cold scan within ~1.5x of dut on the same corpus, with full stats retained.
2. Warm re-run (snapshot load + revalidation) well under 1s for 500k entries.
3. ~25-32 bytes per file record.
4. --help complete enough that an agent needs no other docs; JSON schema versioned and stable.
5. Benchmark harness reports the full cold/warm x raw-walk/with-stats matrix and README claims cite it.
6. Goals 6 and 7 ratified or amended.

Excludes content-tier metrics and the delta journal: both have reserved places in the design, neither is built until the stat tier is solid.
