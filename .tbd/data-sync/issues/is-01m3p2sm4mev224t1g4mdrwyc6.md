---
type: is
id: is-01m3p2sm4mev224t1g4mdrwyc6
title: "Q1: count consumer busy and blocked time, patterns tested per entry, and the freshness pass"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T07:59:15.092Z
updated_at: 2026-09-29T07:59:15.092Z
---
Overnight plan Q1. Add counters: consumer busy vs blocked-in-recv time in scan_concurrent_detached (and the summary consumer), patterns tested per entry and bucket hits (for H171), set_initial_freshness time inside detached_finish_us. Placebo: counters off, instrumented build vs e5a71c8a on default-tree includes zero. Note the counter-bump distortion (424-byte Cell<Counts> copy per bump, counters.rs:859-887): counter-on wall times are not comparable with counter-off runs.
