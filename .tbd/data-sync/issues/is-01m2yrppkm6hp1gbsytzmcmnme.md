---
type: is
id: is-01m2yrppkm6hp1gbsytzmcmnme
title: Reconciliation closure overwrites budget and removes metadata-error entries
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T06:40:21.618Z
updated_at: 2026-09-20T06:40:21.618Z
---
At release integration base 7de72376, opened budget-stop tests end as Partial(Inaccessible) because finish_reconcile overwrites the already-committed Partial(Budget) state when the filesystem walk completed but application hit the file budget. The serial reconciliation listing path also removes entries whose names were enumerated when the following metadata lookup fails, contradicting the retained last-known-fact contract and existing regression. Preserve the specific incompleteness cause and retain enumerated entries while marking failed paths partial.
