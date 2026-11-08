---
type: is
id: is-01m6yeern42yvgewasj4c8bfzs
title: Make the progress-line ordering tests independent of thread scheduling
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-11-08T20:44:03.108Z
updated_at: 2026-11-08T20:44:03.108Z
---
crates/fdu/src/cli.rs run_until_drawn (used by the_line_is_erased_before_an_error and siblings) walks a 1,000-file wide_tree with a ticker at delay 0 / tick 1 ms and fails if no frame was drawn in five runs. Whether the ticker thread is scheduled during so short a walk is up to the OS: on macos-latest it failed on PR #208 head 3c2c0db2 (job 128855087267, 'the ticker drew no frame in five runs') after passing on c8c32306 with the same executed code (3c2c0db2 changed only Cargo.lock's pulldown-cmark, which this metadata-only report never calls). Give the test a deterministic first frame, e.g. a ProgressIo seam that blocks the walk until the ticker has drawn once, or have the ticker draw its first frame synchronously at start under delay 0, rather than retrying and hoping. Do not skip or loosen the ordering assertion.
