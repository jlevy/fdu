---
type: is
id: is-01m3gvqwswcvwe38v0pp58sny0
title: Six-hour cross-platform performance iteration from current main
kind: task
status: closed
priority: 1
version: 6
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
updated_at: 2026-09-27T09:34:22.498Z
started_at: 2026-09-27T07:19:56.353Z
closed_at: 2026-09-27T09:34:22.486Z
close_reason: "Completed the bounded iteration in PR #137. Astra-guided H152 added an exact report oracle; H153 kept a platform-neutral one-pass metric resolution cut with Darwin wall -47.01% and component -59.94%. Rejected the retained-vector variant for +29.58% minor faults. Published exp-158/159 and refreshed all evidence. make check, make cross-lint, and the full GitHub Linux/macOS/Windows matrix pass. Follow-ups fdu-wbhe (Linux replication) and fdu-83wn (post-H153 profile) remain open."
resolution: null
duplicate_of: null
---
Run a bounded six-hour performance loop from main at 4c4917f4. First reconstruct the current Darwin and Linux evidence, obtain an independent Astra review of new hypotheses, then execute the highest-value unattended-safe experiments one at a time. Prefer algorithmic or representation wins likely to transfer across Linux and macOS. Keep source in a dedicated worktree; direct TMPDIR, CARGO_TARGET_DIR, UV_CACHE_DIR, run JSON, and other disposable outputs to /Volumes/spud-ext1/agent-scratch/fdu-perf-6h-20260927. Respect the 3% paired gate, exact oracles, nominated subjects, quiet-host rules, existing dead ends, and person-gated boundaries. Record every valid experiment, regenerate the ledger/report, keep accepted changes, revert rejected code, and hand off through a focused PR without merging.

## Notes

Astra prioritized repeated multi-view content resolution. H152 landed exact outside-timer report oracle (1ba06b19). H153 accepted one-pass shared content/classification resolution (d0902cfd): Darwin content-query wall -47.01% [-47.49%, -45.23%], component -59.94%, exact reports, RSS/minor faults non-inferior. Rejected retained-vector shape at +29.58% minor faults. Evidence exp-158/159 published in a94bac37. make check and make cross-lint pass. Linux replication fdu-wbhe and post-H153 profile fdu-83wn remain open. Awaiting PR/CI before close.
