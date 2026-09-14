---
type: is
id: is-01m2exb1z54reakkxnbzn8zjtg
title: The fdu crate's own tests and clippy do not build without the watch feature
kind: bug
status: closed
priority: 3
version: 4
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T02:53:31.986Z
updated_at: 2026-09-14T21:53:09.548Z
closed_at: 2026-09-14T03:54:42.816Z
close_reason: "8961280: lib-only, in CI and make, now runs clippy -p fdu --no-default-features --all-targets with -D warnings in place of the check, plus the fdu lib tests in that shape. The watch-only tests and imports this bead names were already gated by 078f6fa on #48. CI run 34803813981 ran both steps green."
resolution: null
duplicate_of: null
---
Found while fixing fdu-1onj on PR #51 (2237a70); pre-existing on main (b75bf85), not introduced by the stack.

`cargo clippy -p fdu --all-targets --no-default-features -- -D warnings` fails, and `cargo test -p fdu --no-default-features` does not compile the lib test target. crates/fdu/src/cli.rs@2237a70: the unit test an_interval_parses_without_overflowing_any_platforms_clock (cli.rs:1703-1708) and the watch-rule test (cli.rs:1684-1699) call parse_duration and STYLE_WATCH_RULE, which are #[cfg(feature = "watch")] (cli.rs:1220 parse_duration), and without watch the imports Provenance (cli.rs:23), open_with_pending_save (cli.rs:29), and std::time::UNIX_EPOCH (cli.rs:1675) are unused, while STYLE_WATCH_RULE (cli.rs:73), WATCH_SCOPE_VOCABULARY (cli.rs:1074), and watch_scope_guidance (cli.rs:1093) are dead. A plain `cargo build -p fdu --no-default-features` still succeeds with warnings.

crates/fdu/Cargo.toml says the command line still builds and works without watch, but CI never exercises that shape for the fdu crate (ci.yml runs --no-default-features only for fdu-core), so it rots unseen, the same way fdu-core's did before its feature-boundary job.

Fix direction: gate the watch-only tests and imports on the feature (or move them under a watch-gated test module), then add `cargo clippy -p fdu --all-targets --no-default-features` or `cargo test -p fdu --no-default-features` to the feature-boundaries CI job so the Cargo.toml claim is checked.

## Notes

2026-09-14 (fdu-x7yb, PR #60, 077318c): the gitignore build feature is removed, so the `fdu` crate's `--no-default-features` shape now drops only the `watch` build feature. The `cargo clippy -p fdu --no-default-features --all-targets` and `cargo test -p fdu --no-default-features --lib` steps this bead added are unchanged in lib-only and the test-lib-only CI job.
