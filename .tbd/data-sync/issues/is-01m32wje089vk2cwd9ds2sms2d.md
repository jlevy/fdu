---
type: is
id: is-01m32wje089vk2cwd9ds2sms2d
title: Attribute one-shot scan errors per directory so --allow-partial rows are not all incomplete
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-21T21:04:53.768Z
updated_at: 2026-09-21T21:04:53.768Z
---
Follow-up from the PR #96 R1 disposition (https://github.com/jlevy/fdu/pull/96#issuecomment-5765069350). A one-shot scan that finishes with errors records that the walk was partial (set_initial_freshness(false)) but not which directory each error fell in: ScanReport.errors are rendered to strings and never reach the Index, so under --allow-partial every directory row reports complete: false and an unknown age. Plumb the failing paths (ScanReport.errors carry them) into the index or Provenance so only the directories the errors fall in, and their ancestors, are marked incomplete; an opened root already gets this from children_complete. Acceptance: a fixture with one unreadable directory under --allow-partial marks that directory and its ancestors incomplete and leaves siblings complete.
