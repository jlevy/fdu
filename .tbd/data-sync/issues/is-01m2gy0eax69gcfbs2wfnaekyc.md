---
type: is
id: is-01m2gy0eax69gcfbs2wfnaekyc
title: reconcile keeps take-then-reassign where retain would edit in place
kind: task
status: open
priority: 4
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T21:43:41.656Z
updated_at: 2026-09-14T21:43:41.656Z
---
Guideline conformance review of PR #56, finding 6 (https://github.com/jlevy/fdu/pull/56#pullrequestreview-5203155693). scan.rs:848 and :4014 at cfd1335 take a Vec out, filter it and reassign it; Vec::retain edits in place and says so (rust-rules: avoid allocation-forcing patterns). Low priority.
