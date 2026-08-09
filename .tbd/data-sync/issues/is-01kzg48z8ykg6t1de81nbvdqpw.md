---
type: is
id: is-01kzg48z8ykg6t1de81nbvdqpw
title: "Spike: revalidation cost curve at 500k entries"
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - phase1-foundation
dependencies:
  - type: blocks
    target: is-01kzg4ak7v8z2a7s41rsms8jcb
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:26:52.701Z
updated_at: 2026-08-09T20:36:55.746Z
---
THE load-bearing assumption of the cache design: a parallel truth-check of 500k unchanged entries is fast enough to feel instant. Build on the shared validated corpus and runner from fdu-rq5m/fdu-d8kq. Measure 10k/100k/500k/1M curves for the current full sweep and directory-mtime shortcut, naming snapshot state and filesystem-cache state independently: uncontrolled for ordinary local runs, verified-warm for prepared runs, and controlled-cold only on a documented dedicated host. Report snapshot load, revalidation, any snapshot rewrite, and product completion separately. If the 500k target fails, revise cache tiering before freezing the snapshot format; do not hide the result in one favorable number.
