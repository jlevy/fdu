---
type: is
id: is-01m2ex9y978m63k9yzdt5esfqa
title: Compact [[kind]] manifest reader misreads trailing comments, commas in strings, and escapes
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T02:52:55.462Z
updated_at: 2026-09-14T03:54:31.358Z
closed_at: 2026-09-14T03:54:31.357Z
close_reason: "f4a7131: the compact [[kind]] reader now reads with the File Rollup registry's TOML cursor, moved to classify/manifest_toml.rs and included by build.rs beside the parser, so both dialects accept the same TOML and refuse the same forms by name. The registry's form table is added for the compact dialect and fails on the old reader. The default manifest fingerprint is unchanged."
resolution: null
duplicate_of: null
---
Found while fixing fdu-m5zj (PR #48 review CLASS-7) at 4da6d60. That fix replaced the File Rollup registry reader. The compact `[[kind]]` dialect has the same defect and was left alone.

`crates/fdu-core/src/classify/type_rule_manifest.rs:123-140` (at 4da6d60): `parse_manifest_string` strips one `"` from each end of the trimmed text after the first `=`, and `parse_manifest_array` splits on every `,`. `TypeRegistry::from_manifest` reads caller-supplied compact manifests at runtime through this code, not only the build-time `rules/file-types.toml`, so a caller can hit these misreadings:

- `id = "notes" # "x"` reads as the id `notes" # "x` (it then fails id validation, so the error names the wrong thing).
- `extensions = ["md"] # ]` reads `"md"] # ` as an item and fails with "expected a quoted string".
- `extensions = ["a,b"]` splits inside the string.
- An escaped `\"` stays literal.
- A BOM, a multi-line array, or a literal string is rejected with a generic message.

Fix: read compact manifests with the cursor 4da6d60 added to `file_rollup_manifest.rs`, or share its value reader, so both dialects accept the same TOML and refuse the same forms by name. `build.rs` `include!`s this file (`crates/fdu-core/build.rs:24`), so the reader must stay dependency-free and compile in the build script. Add the same form table as `file_rollup_manifest::tests`.
