---
type: is
id: is-01m32f6qyw26btzdt7de1m4hrs
title: Detector recipes for signal-cancelling tests, from the 2026-09-21 sweep
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:11:19.260Z
updated_at: 2026-09-21T17:11:19.260Z
---
Seven places worth pointing a systematic detector, ranked, from the sweep that found fdu-fvcx and fdu-hb2t.

1. Hand-set ceilings of the form `assert!(growth <= n * K)`. Print the measured/ceiling ratio and require that a +1-per-unit mutation crosses it. That recipe found fdu-hb2t: opened slope 24.33 vs ceiling 26 passes a one-per-entry regression, while the detached counterpart (6.15 vs 7) correctly fails.
2. Tests named `*_miss`, `*_hit`, `*_skips`, `*_clean_miss` whose only assertions are `== default()`, `hits ==` or `applied ==`. Require a sibling assertion on the ANSWER under the mismatched request.
3. Waiver and classifier tables in gates (`parity-classes.mjs`, and any future `CLASSES`). Forbid length-only matches; unit-test each class with a "one real change added" negative case.
4. `-> bool` functions named serve/usable/reuse/compat carrying more than one `&&` chain. Require a value-carrying enum instead: `Serves` gains a `Project` variant, `ContentCacheLoad.usable` stops being a bool.
5. Identity structs with several fields derived from one input (`analysis`, `options_fingerprint`, `provenance.analyzers`). Partial relaxations are masked — see fdu-82vm.
6. Files carrying `#[global_allocator]` or calling `counters::enable`. Enforce exactly one `#[test]` per binary, in the style of `check-admission-sites.mjs`. Today's cleanliness rests on `counters::test_serial()` being taken at all 11 in-crate sites and on both integration binaries happening to have one test each; `test_serial` is `pub(crate)`, so a second `#[test]` in an integration binary recreates the torn instrument with only a file comment as defence.
7. Goldens: track the per-file `--cache off` ratio and require at least one case per cached subsystem where a DIFFERENT request follows a warm cache. Current ratios: cli-axes 38/45, cli-content 26/36, cli-human 7/7, cli-json 4/4, cli-overview 1/1, cli-surface 5/11, cli-cache 1/17, cli-lifecycle 0/38, cli-watch 0/6. No golden issues a different analyzer set against a warm sidecar, which is exactly the fdu-gija shape.

Also worth recording as tautologies found and judged harmless rather than fixed: `scan.rs:6464 wall_ns >= work_ns` (wall encloses work), and `scan.rs:5632 window.active_workers <= peak_active_workers` (peak is an atomic max over active, so inert by construction). The real disjointness check is `accounted_ns() <= wall_ns` at `:6449`.
