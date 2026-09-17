---
type: is
id: is-01m2rgb3n980wsz88da6d81ndk
title: "PR #82 review F1: A warm partial pass rewrites the sidecar with only the changed files"
kind: task
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2rgb3631yn1waftyaj31wca
created_at: 2026-09-17T20:18:46.568Z
updated_at: 2026-09-17T20:18:46.568Z
---
PR #82 review (https://github.com/jlevy/fdu/pull/82#issuecomment-5720664716), finding F1.

Every later run re-reads the whole tree while a directory is unlistable, because unchanged entries stay Cached and a partial pass has no Fresh subtree until P1.4.2. Gate the partial-pass sidecar write until P1.4.2 marks failed paths, and pin it with a test.
