---
type: is
id: is-01m0y1sfw7kwjprd6sfky281fj
title: Revise the MetaBrowser provider contract from measured evidence
kind: feature
status: in_progress
priority: 1
version: 15
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sg8emg0sgyv1pj8sa6x7
  - type: blocks
    target: is-01m0y1sgqd1sd33stssgw25f2q
  - type: blocks
    target: is-01m0y1shykye8sc7h7e9rkk6kh
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
  - type: blocks
    target: is-01m10nr9wf12chcxvgv2qjs4qr
  - type: blocks
    target: is-01m10nrazfqj0ndxdpvv94kprg
  - type: blocks
    target: is-01m10nrc1rnh7e8zzwx0z8r76c
  - type: blocks
    target: is-01m10nsdjx7z9h87m4nf8hzhyh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nr8phjnfwhjak56e360gw
  - is-01m10nr925xb4ybt40q2pw7zpn
  - is-01m10nr9f2mkwdtp8ad88ms621
created_at: 2026-08-26T03:28:32.134Z
updated_at: 2026-08-28T02:03:00.668Z
---
Update contract.py and its conformance registry: registry document input, DiscoveryBudget execution policy, max depth as selection, explicit scope values, derived identities, exhaustive state vocabulary, exact tree/flat order, portable-path issues, work limits, opaque pages without remaining_rows, and exact-or-capped totals.

## Notes

## MetaBrowser-side ordering work, decided 2026-08-27

The three row orders are now stated in the fdu plan under "Row order is stated here, not
inferred from an implementation", and the required MetaBrowser edits are listed under
"Provider contract values". Recorded here so the decision is not lost between repos.

1. Write the three orders into `contract.py` beside the dataclasses they govern:
   breadth-first level order for tree and directory pages, canonical POSIX path order for
   flat and catalog pages, and `(mtime descending, canonical POSIX path ascending)` for
   ranked recent rows. `contract.py` at `45266a8` documents none of them.
2. Delete the ranked-recency reordering in `_recent_projection` that moves ignored
   entries behind unignored ones when the match count exceeds the row bound. It fires in
   one branch and not the other, so one query name carries two ranking contracts.
   `include_ignored` filters; it does not rank.
3. State that `include_ignored: false` prunes the excluded directory's whole subtree.
   `_directory_rows` already prunes by skipping before extending its frontier; the
   contract has to say so, because filtering rows is an equally reasonable reading of an
   unstated rule.
4. Confirm level order is intended rather than incidental. `_directory_rows` implements
   it with an explicit frontier, and fdu now matches it deliberately.

The conformance packet gains order cases that a tie alone can fail: a tree deep and wide
enough to separate level order from pre-order, interleaved directory and nondirectory
names, files sharing one modification time, an ignored directory holding the newest file,
and the same query run over one corpus that overflows the row bound and one that does not.
