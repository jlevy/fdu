---
type: is
id: is-01m3qgck5yzpd603akhhkpw7t9
title: "Parallel track: make fdu uniformly faster than pdu (every subject, every pdu mode, Linux and macOS)"
kind: epic
status: open
priority: 1
version: 10
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
updated_at: 2026-09-30T05:09:04.338Z
---
Maintainer request 2026-09-29: an aggressive, parallel investigation of how fdu can be uniformly faster than pdu, run on top of the stabilizing #157/#158/#161/#162 stack without blocking it. Standing at the stack's head (2026-09-29, Linux 4-vCPU guest): level with pdu's default on linux-v6.12 (+1% [-2%, +2%]) and node-modules-dense (+1% [-2%, +4%]), 20 pairs each; on the generated 1M balanced tree fdu's default tree is 3.6% faster than pdu's default but pdu --max-depth 2 is 2.5% [1.4%, 4.0%] faster than fdu. Goal: a measured, pre-registered path to fdu ahead of every pdu mode on every nominated subject, with side-by-side profiles attributing each remaining gap. Branch claude/pdu-uniform-lead, stacked on #162; registry ids H185+ and exp-197+ (exp-196/H184 are reserved for the autofs fix fdu-d2fn).

## Notes

2026-09-30: measured on a quiet host, exp-197 to exp-201. H185 and H186 accepted on node-modules-dense (not resolvable on linux-v6.12), H188 with H189 accepted, H187 rejected and reverted. The branch claude/pdu-uniform-lead ships engine a356d456. exp-201: shipped default tree -3.05% (linux-v6.12) and -8.94% (node-modules-dense) against the round's final head; in the same run fdu's default leads pdu default by 13% and 15%, diskus by 12% and 11%, pdu --max-depth 2 by 3% [+1%, +8%] and 10% [+4%, +13%]; about six points of the lead over pdu default are the regime. Remaining: macOS cells (exp-202+), walker-side levers (H169 phase 3, H177), H178.

2026-09-30: the maintainer confirmed the release scope: 0.3.0 = #157 -> #158 -> #161 -> #162, then the stability PR (claude/stability-fixes with claude/stability-tooling merged in, epic fdu-l4u1), then the pdu track (claude/pdu-uniform-lead, epic fdu-faqa) on top, as one linear stack merged bottom to top with merge commits. Still open for the maintainer before tagging: fdu-8f6k (#[non_exhaustive] Counts), fdu-q7hf (reader diagnostics fields in 0.3.0), fdu-4nue (JSON default view).
