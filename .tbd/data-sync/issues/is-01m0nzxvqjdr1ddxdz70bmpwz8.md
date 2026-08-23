---
type: is
id: is-01m0nzxvqjdr1ddxdz70bmpwz8
title: "PR #42 review R6: emit_version duplicated verbatim across both build scripts"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:02.738Z
updated_at: 2026-08-23T00:39:54.885Z
closed_at: 2026-08-23T00:39:54.885Z
close_reason: "Fixed differently than proposed. Extraction is not available: a cross-crate include! names a file outside the including crate's package, so it would not reach the published .crate and the packaged source would not compile -- the same failure that shipped fdu without its build script. A third crate for twenty lines is worse. Both copies are now delimited by BEGIN/END markers explaining why they are duplicated, and tests/release/test_metadata.py fails if they drift. Verified red-green."
---
crates/fdu/build.rs:24 and crates/fdu-core/build.rs:51 hold ~45 identical lines and five git subprocesses each. Both copies are needed: benchmarks/realtree/provenance.py::_fdu_revision_reasons asserts g<sha> appears in the probe's --version. Extract rather than delete.
