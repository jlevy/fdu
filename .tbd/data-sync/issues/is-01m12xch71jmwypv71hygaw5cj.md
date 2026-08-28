---
type: is
id: is-01m12xch71jmwypv71hygaw5cj
title: Settle the joint Recent ordering contract before wiring the maintained recency index
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-28T00:47:45.369Z
updated_at: 2026-08-28T00:47:45.369Z
---
The maintained global recency index (ServingIndexes::recent_files) is built, mutation-tested against independent recomputation, and has NO reader. Wiring it is blocked on a joint ordering decision, not on fdu implementation.

fdu's RecentKey defines a total order: mtime descending, then portable_path ascending, then EntryId. The MetaBrowser reference provider at 45266a8 (_recent_projection in providers/python_inventory.py) instead does:

    matching.sort(key=lambda entry: entry.mtime_ns, reverse=True)

a stable sort on mtime alone. Ties therefore resolve to image.entries order, which is implementation-defined rather than contractual. Two providers cannot be proven to agree on a tie under fdu-xu27's replay conformance while the contract leaves ties unspecified.

A second inconsistency in the same function needs a decision with it. When total > max_rows the reference deprioritizes gitignored entries regardless of mtime, but when total <= max_rows it returns the plain mtime order. So the top N is 'the N newest' in one branch and 'the N newest preferring unignored' in the other. A caller cannot tell which contract it got, and fdu has no equivalent rule.

A third point to settle: fdu's recency index holds only representable files, while a walk sees every entry. On a tree with unrepresentable names the two populations differ, so any fdu fast path must either prove the portable index complete or fall back.

Decide, jointly with MetaBrowser, and record in the plan:
1. the total tie-break order for recent rows, canonical POSIX path being the natural candidate since it already orders flat and catalog pages;
2. whether ignored deprioritization is part of the contract at all, and if so in both branches;
3. the required behavior when the portable index is incomplete.

Only then wire recent_files into the opened-root read path with a maintained-versus-recomputed oracle. Do not guess an order in fdu: a wrong guess is invisible until the cross-provider replay, which is exactly the failure mode the rewrite exists to prevent.

Related: the navigation aggregates raise the same question for the classification views, which currently charge a full pass in report_work while semantic_by_directory is maintained.
