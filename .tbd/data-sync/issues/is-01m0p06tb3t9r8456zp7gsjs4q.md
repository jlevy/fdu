---
type: is
id: is-01m0p06tb3t9r8456zp7gsjs4q
title: "PR #42 R6: emit_version duplicated verbatim across two build scripts"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:56.226Z
updated_at: 2026-08-23T00:57:54.488Z
closed_at: 2026-08-23T00:57:54.488Z
close_reason: "Rebutted, not fixed: env! expands in the crate that reads it, so the emit_version duplication cannot be removed without breaking cargo package (a shared pub const was tried and fails verification), a shared include! file would sit outside both packages, and a shared build-dependency would be a third crate. Reasoning recorded at both copies in 4e34ce3."
---
crates/fdu/build.rs:24 and crates/fdu-core/build.rs:51. ~45 identical lines, five git subprocesses, in two places. Both need it: the perf-provenance gate asserts the stamp in the probe version.
