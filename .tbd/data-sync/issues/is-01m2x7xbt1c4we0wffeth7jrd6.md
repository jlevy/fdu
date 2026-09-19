---
type: is
id: is-01m2x7xbt1c4we0wffeth7jrd6
title: Remaining hypotheses after H115+H120 overnight
kind: epic
status: open
priority: 1
version: 21
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
child_order_hints:
  - is-01m2x7xnqp8jhj3qjf6m2bh0qy
  - is-01m2x7xpapt6wnpmg5rfqfm0x7
  - is-01m2x7xn536s70m3z293rqkyhk
  - is-01m2x7xpxrbgq7sfs6501vn5z6
  - is-01m2vseyr1w2b7e4sbyxxckx2g
  - is-01m2xp6x0akn2wfj0p4rx7zp1k
  - is-01m2xptsdmc6vtbekr1xtwfjgh
  - is-01m2xqbkanyh7vdfmbzdstxkkq
  - is-01m2xqywv4ssfs89346n79jgz1
  - is-01m2xrgf9vg6pexb6cjvf92v3f
  - is-01m2xs3szwzwsyk5s25myn18hp
  - is-01m2xt60nz02xa4wnkbxspfg75
  - is-01m2xtkpsdv305jn0fmgw9hh1c
  - is-01m2xvc4e8rqqfzepze97j7fnc
  - is-01m2xvqc370b593ksygakk5w9z
  - is-01m2xwkb23n85d9n7tnhjmetg5
created_at: 2026-09-19T16:27:39.712Z
updated_at: 2026-09-19T22:29:11.362Z
---
Morning registry pass after the H116-H120 overnight. Owns the remaining unaddressed queue: H113 (quiet, existing fdu-rfr6), H121-H124 (new), H107 ignore-is-the-walk only (fdu-jcfn), H111 not-in-this-host (fdu-jekg). Spec is the one source of truth; runbook standing points here. Docs and beads only; do not start a measurement cell. Do not retry H116/H118/H119, H114, H109, snapshot-load, or the H86/EntryId rewrite.

## Notes

H122 leftover exp-122 recorded. H113 leftover exp-123 recorded: completeness walk still 16% of content_open. H125 not minted. Remaining on this host: H113 needs quiet (gates 69.4%, 43.79%, 85.17%); H107 has no ignore-is-the-walk subject; H111 not in this host. Next experiment id after leftover is exp-124 (exp-113 still reserved).
