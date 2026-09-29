---
type: is
id: is-01m3pf4h3wj4hjd0fzfkhcg3yk
title: "H182: folded-tier listing sort by (fnv32(name), position), name order restored once in finish"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T11:34:55.356Z
updated_at: 2026-09-29T11:34:55.356Z
---
From the Fable mid-night sweep. Under the H172 tier the consumer still sorts each listing by name with memcmp (index.rs ~1813-1829, ~1920-1949; 308/259 instructions per entry on linux-v6.12/node-modules-dense). Sort by (fnv32(name), position) with u32 compares, dedup adjacent by hash then bytes, allocate non-files in that order, and restore name order once per gained directory in finish (which keep_largest_files already re-sorts). Exact because a folded index never escapes the one-shot route (H172 review). Predicted -1.5 to -2 ms linux-v6.12, -1 to -1.5 node-modules-dense. Bundle cell with H181.
