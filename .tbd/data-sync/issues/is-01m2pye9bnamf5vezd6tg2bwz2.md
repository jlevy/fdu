---
type: is
id: is-01m2pye9bnamf5vezd6tg2bwz2
title: "P2.2.1: The emit module (Sink, JsonSink, YamlSink) and the YAML scalar policy"
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye9p8df0h0rzch4fq37wy
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
hold: null
hold_until: null
created_at: 2026-09-17T05:46:41.908Z
updated_at: 2026-09-20T04:35:01.882Z
started_at: 2026-09-20T04:35:01.881Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `emit.rs`, `emit/emit_json.rs`, `emit/emit_yaml.rs`, `emit/emit_scalar.rs` (new): `trait Sink { begin_map(Shape), end_map, begin_seq(Shape), end_seq, key(&'static str), str, u64, i64, bool, null }` with `enum Shape { Block, Inline }`; `JsonSink::{pretty, line}`; `YamlSink`; `is_plain_safe`, `write_json_string`, `write_yaml_scalar`. Zero dependencies; generic, not `dyn`, so scalar calls monomorphize.
- Scalar policy (fdu-4xy9): plain only for non-empty ASCII `[A-Za-z0-9._/+-]` that does not start with a digit or sign, is not dot-numeric or `.inf`/`.nan`, and is not a YAML 1.1 boolean or null spelling; otherwise double-quoted, escaping `"`, `\`, controls below 0x20, 0x7F-0x9F, U+2028, U+2029, U+FEFF, U+FFFE, and U+FFFF.

**Tests**

- The scalar policy over the 121-string corpus from `explorations/yaml-conformance`, copied to `crates/fdu-core/src/testdata/`.
- The JSON escape set; sink nesting and errors; a `SchemaCheck` adapter checking key order and presence against `Field` tables.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
