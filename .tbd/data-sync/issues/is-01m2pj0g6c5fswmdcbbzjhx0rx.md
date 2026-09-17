---
type: is
id: is-01m2pj0g6c5fswmdcbbzjhx0rx
title: One ordered value model rendered by the JSON and YAML serializers
kind: task
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - release
dependencies: []
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:09:27.242Z
updated_at: 2026-09-17T02:57:34.062Z
---
Build each machine document once and render it through generic serializers, replacing the parallel
per-field writers for reports, watch changes and cache status. Byte-stable goldens for JSON; YAML shape
follows JSON by construction.

## Notes

2026-09-17 design from the evaluation. A `yaml_emit` module in fdu-core, promoted to a first-party crate
when a second consumer appears:
  pub fn is_plain_safe(s: &str) -> bool  // ASCII [A-Za-z0-9._/+-]; not digit- or sign-leading; not dot-numeric; not true/false/yes/no/on/off/y/n/null/~
  pub fn write_scalar(out: &mut impl fmt::Write, s: &str) -> fmt::Result      // plain or double-quoted
  pub fn write_json_string(out: &mut impl fmt::Write, s: &str) -> fmt::Result // same escape set
  pub trait Sink { begin_map, end_map, begin_seq, end_seq, key, str, u64, i64, bool, null }
  pub struct YamlSink<W>; pub struct JsonSink<W> (pretty and one-line JSONL modes)
One streaming walk per document writes to either sink, so JSON and YAML cannot diverge and no per-field
allocation is added for 300k-row reports. Port report, watch-change and cache-status output.
