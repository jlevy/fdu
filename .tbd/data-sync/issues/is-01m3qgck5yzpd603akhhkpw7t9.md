---
type: is
id: is-01m3qgck5yzpd603akhhkpw7t9
title: "Parallel track: make fdu uniformly faster than pdu (every subject, every pdu mode, Linux and macOS)"
kind: epic
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
child_order_hints:
  - is-01m3qk1dx951we3ftrs668xr0w
  - is-01m3qk1ecctt5q77ye65njtxe2
  - is-01m3qk1ev3m8tws4t0vb1n8p5v
  - is-01m3qk1f9s3gfa3k26pnw8688q
  - is-01m3qk1fr6exhnjzd8c8xbcark
  - is-01m3qk60n034qgmzf388214bmt
  - is-01m3qrhfc55cdy67aaetregzck
created_at: 2026-09-29T21:16:02.621Z
updated_at: 2026-09-29T23:38:31.173Z
---
Maintainer request 2026-09-29: an aggressive, parallel investigation of how fdu can be uniformly faster than pdu, run on top of the stabilizing #157/#158/#161/#162 stack without blocking it. Standing at the stack's head (2026-09-29, Linux 4-vCPU guest): level with pdu's default on linux-v6.12 (+1% [-2%, +2%]) and node-modules-dense (+1% [-2%, +4%]), 20 pairs each; on the generated 1M balanced tree fdu's default tree is 3.6% faster than pdu's default but pdu --max-depth 2 is 2.5% [1.4%, 4.0%] faster than fdu. Goal: a measured, pre-registered path to fdu ahead of every pdu mode on every nominated subject, with side-by-side profiles attributing each remaining gap. Branch claude/pdu-uniform-lead, stacked on #162; registry ids H185+ and exp-197+ (exp-196/H184 are reserved for the autofs fix fdu-d2fn).
