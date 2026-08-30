---
type: is
id: is-01m18r51dyvcp3bzw8yca45ph7
title: Control state does not scale to a real home directory
kind: epic
status: open
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - macos
  - control-state
dependencies: []
child_order_hints:
  - is-01m18r5z4kjptahbmkx2ez939k
  - is-01m18r5zf7hy5bqc66pkhpa58n
  - is-01m18r5zsmxvm2a098rh56w9yr
  - is-01m18r6049rg359vn5nr1tazky
  - is-01m18r70ah7yekzdr3525x8jky
created_at: 2026-08-30T07:11:43.549Z
updated_at: 2026-08-30T07:12:47.952Z
---
Agent field reports: fdu 0.1.0-dev+g27aeed0ef.dirty (branch codex/opened-root-inventory-rewrite, PR #48) cannot complete a roll-up of ~ or ~/wrk on macOS. Two scans abort with 'control table requires N bytes; limit is 4194304 bytes'; a third (~/Library) is SIGKILLed (137).

Verified on this machine:
- fdu ~/wrk on that binary: 'control table requires 4203751 bytes; limit is 4194304'.
- main has no control.rs and scans ~/wrk in 15s, exit 0. The defect is branch-only and would land on main when PR #48 merges.

Correcting the field report: the overshoot is NOT ~0.4%. The limit trips at the first crossing of a running cumulative total, so the reported number is where it crossed, not what it needed. Computing the branch's own retained_source_cost over all 3256 .gitignore files in ~/wrk gives 9.93 MiB charged against a 4 MiB cap - 2.4x over. Raising the cap to 5-8 MiB looks sufficient and is not.

This epic collects the scale, bound-discipline, and diagnosis work required before PR #48 can leave draft.
