---
type: is
id: is-01m0nrzwe28fz1esdk2nnxd0eg
title: Move the CLI's tests with it, and rehome the test-only helpers
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-22-fdu-cli-on-the-public-api.md
labels: []
dependencies: []
parent_id: is-01m0nrykr3me8qck91cp0ydhnn
created_at: 2026-08-22T22:20:48.961Z
updated_at: 2026-08-22T22:55:20.553Z
closed_at: 2026-08-22T22:55:20.553Z
close_reason: "crates/fdu-cli depends on fdu as an ordinary crate, so the boundary is enforced by the compiler: cli.rs has zero crate:: paths, the library has no cli module or feature and no clap or anyhow, and make lib-only fails if either returns. All 129 goldens byte-identical, none regenerated."
---
cli.rs carries its own #[cfg(test)] module, which reaches three pub(crate) items that production does not: apply_ok, set_initial_freshness, and view_label.

view_label is ViewSpec::label now, so that one is a substitution. The other two are test support and either move with the tests, become test-only public API behind a feature, or are replaced -- decide per item.

Files: the tests module in crates/fdu/src/cli.rs, crates/fdu/src/test_support.rs.
