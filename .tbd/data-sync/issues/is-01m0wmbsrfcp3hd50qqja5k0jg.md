---
type: is
id: is-01m0wmbsrfcp3hd50qqja5k0jg
title: Implement exact MetaBrowser catalog predicate semantics
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019981640
    at: 2026-08-25T14:15:45.882Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T14:14:37.582Z
updated_at: 2026-08-25T14:15:45.883Z
---
At PR 47 exact head 1e9b85d4ce6b4c01fa800f8a25eb607ebb9675a0, the reference catalog_page proves only an unconstrained all-files page. MetaBrowser CatalogQuery also requires case-insensitive terminal-extension matching, exact case-sensitive ancestor-component matching, include_ignored, and an exclusive size upper bound. FDU Selection can exactly translate the ignored tag and exclusive size bound, but its generic case-sensitive path/name globs are not equivalent to the two remaining predicates. Add closed native predicate fields or another demonstrably exact engine-side translation, expose them through Python, and run them through resumable paging and a shared provider/conformance fixture. Do not filter rows in Python or create a mirror index.
