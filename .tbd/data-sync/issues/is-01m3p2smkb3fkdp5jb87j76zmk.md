---
type: is
id: is-01m3p2smkb3fkdp5jb87j76zmk
title: "H174: walker-side listing digest (sort, extension buckets, per-listing tallies computed by the walker; consumer stays the single writer)"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T07:59:15.563Z
updated_at: 2026-09-29T07:59:15.563Z
---
Overnight plan Q5 (new hypothesis). Per-entry consumer work that depends only on one listing moves to the walker that read it before sending: name sort + dedup, each file's extension bucket, the listing's per-extension and total tallies. The consumer interns a few extensions per listing and merges one pre-folded contribution into the parent. Every walker-computed value is a pure function of its own listing, so the single-writer rule holds. Not dut's walker-side roll-up (T4), not H157's in-place allocation trims. Deciding: default-tree on node-modules-dense and linux-v6.12 (on top of H172); consumer instructions -30%; placebo aggregate-summary. Fable design review before code.
