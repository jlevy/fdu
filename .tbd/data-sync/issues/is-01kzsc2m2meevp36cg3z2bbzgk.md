---
type: is
id: is-01kzsc2m2meevp36cg3z2bbzgk
title: Watch integration tests are Unix-only, leaving Windows watch uncovered
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T21:36:23.123Z
updated_at: 2026-08-11T21:36:23.123Z
---
crates/fdu/tests/watch_session.rs and watch_persistence.rs are both gated #![cfg(all(feature = "watch", unix))], so no test exercised --watch on Windows at all. The consequence was found the hard way: parse_duration built an anchor 2^40 seconds past the epoch, which is fine where SystemTime counts seconds and overflows on Windows where it counts 100ns FILETIME ticks, so every fdu --watch invocation panicked on Windows regardless of arguments. It shipped through many green CI runs because the only Windows-visible watch goldens were scope-validation rejections, which exit before the watch path is reached. Fixed by bounding the anchor, with a cross-platform unit test, and the new cli-watch golden now drives a real watch session on Windows CI. Remaining work: decide whether the two integration suites can run on Windows. They use real filesystem events, so ReadDirectoryChangesW semantics (coalescing, rename pairs, directory-level notification) may differ enough to need their own expectations rather than a relaxed cfg. Until then, Windows watch coverage rests entirely on one golden.
