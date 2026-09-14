---
type: is
id: is-01m2f3trcyqwwfspfr7g810f3d
title: The raw-extension docs say any final component counts, but a non-UTF-8 name has no extension
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T04:46:57.949Z
updated_at: 2026-09-14T04:46:57.949Z
---
PR #48 delta review PR48-DOC-1 (https://github.com/jlevy/fdu/pull/48#pullrequestreview-5194007815). crates/fdu-core/src/classify.rs:15, 931 and docs/project/architecture/fdu-engine-architecture.md:383 at d48b8f8, from 0a2e341. The prose says the raw level counts a final component whatever its bytes. derive_ext_native (classify.rs:989, :1010) returns None for invalid UTF-8 or UTF-16, so such a name goes in the (none) bucket. The example table is correct; only the prose is wrong. Fix the prose on both surfaces, and add a row for a non-UTF-8 name if a portable test can pin it.
