---
type: is
id: is-01m2ewjkmv0bvwq81pw9tshd4e
title: The featureless command line's --docs guide names --watch, which that binary does not have
kind: bug
status: open
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T02:40:10.887Z
updated_at: 2026-09-14T02:40:23.968Z
---
Found while fixing fdu-2wlp, at commit 078f6fa on codex/opened-root-inventory-rewrite.

With the watch gates in place, `cargo check -p fdu --no-default-features --all-targets` passes, and so do the featureless integration tests (cli_color, cli_exit, scan_watermark). One lib test fails in that shape:

    cargo test -p fdu --no-default-features --lib the_guide_only_names_flags_that_exist
    --docs names --watch, which is not a flag

The `DOCS` guide text (crates/fdu/src/cli.rs:129, :131 and :140 at 078f6fa) names `--watch` and `--interval` unconditionally. Those flags are compiled out without the `watch` feature. So a featureless binary's `--docs` advertises flags its parser rejects, which is the exact thing that test exists to prevent.

Not fixed in fdu-2wlp because the right answer is a design choice. The guide could drop its watch lines when the feature is off (a cfg-selected or composed DOCS), or say the build lacks watch. Either way the test would then pass in both shapes.

The new lib-only gate is a `cargo check`, so it does not run this test and CI stays green. Once the guide is fixed, consider running `cargo test -p fdu --no-default-features --lib` in lib-only as well.
