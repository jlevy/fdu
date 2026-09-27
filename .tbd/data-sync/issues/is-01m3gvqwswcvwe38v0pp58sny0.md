---
type: is
id: is-01m3gvqwswcvwe38v0pp58sny0
title: Six-hour cross-platform performance iteration from current main
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - performance
  - campaign-2
  - overnight
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
child_order_hints:
  - is-01m3gw0t783gp8kn5deq8ctfgv
  - is-01m3gx1m2w5jq4fpbag5gjwd8p
hold: null
hold_until: null
created_at: 2026-09-27T07:19:46.233Z
updated_at: 2026-09-27T09:26:21.084Z
started_at: 2026-09-27T07:19:56.353Z
---
Run a bounded six-hour performance loop from main at 4c4917f4. First reconstruct the current Darwin and Linux evidence, obtain an independent Astra review of new hypotheses, then execute the highest-value unattended-safe experiments one at a time. Prefer algorithmic or representation wins likely to transfer across Linux and macOS. Keep source in a dedicated worktree; direct TMPDIR, CARGO_TARGET_DIR, UV_CACHE_DIR, run JSON, and other disposable outputs to /Volumes/spud-ext1/agent-scratch/fdu-perf-6h-20260927. Respect the 3% paired gate, exact oracles, nominated subjects, quiet-host rules, existing dead ends, and person-gated boundaries. Record every valid experiment, regenerate the ledger/report, keep accepted changes, revert rejected code, and hand off through a focused PR without merging.

## Notes

Astra prioritized repeated multi-view content resolution. H152 landed exact outside-timer report oracle (1ba06b19). H153 accepted one-pass shared content/classification resolution (d0902cfd): Darwin content-query wall -47.01% [-47.49%, -45.23%], component -59.94%, exact reports, RSS/minor faults non-inferior. Rejected retained-vector shape at +29.58% minor faults. Evidence exp-158/159 published in a94bac37. make check and make cross-lint pass. Linux replication fdu-wbhe and post-H153 profile fdu-83wn remain open. Awaiting PR/CI before close.
