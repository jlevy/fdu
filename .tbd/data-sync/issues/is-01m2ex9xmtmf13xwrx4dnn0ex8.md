---
type: is
id: is-01m2ex9xmtmf13xwrx4dnn0ex8
title: MetaBrowser's shared registry is File Rollup schema 4, which the engine rejects
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-14T02:52:54.808Z
updated_at: 2026-09-14T13:56:26.853Z
---
Found while fixing fdu-m5zj (PR #48 review CLASS-7) at 4da6d60.

MetaBrowser's shared registry, `src/metabrowser/data/file-rollup-format/recommended-file-types.toml`, has been `schema_version = 4` since metabrowser commit 41975d5d7, "feat: the file tree takes its identity from the rollup registry" (2026-08-30). That commit adds `icon` to every `[[group]]` and `[[family]]`. On main it is still schema 4 and `registry_revision = 3`, with 6 groups, 56 families, and 64 kinds.

fdu accepts only schema 3. `crates/fdu-core/src/classify/file_rollup_manifest.rs:21` (`SCHEMA_VERSION`, at 4da6d60) sets that, and the per-table field matches reject `icon` first: `TypeRegistry::from_manifest` fails on MetaBrowser main with `line 34: unknown group field "icon"`. The last schema 3 revision, metabrowser 1e1f5f912, parses with fingerprint 0x951d41f9a873ef78, which matches Python tomllib's reading of it.

So the spec's "MetaBrowser must pass the actual File Rollup registry document at open" (plan-2026-08-25, line 392) cannot hold against MetaBrowser main today. Someone has to decide:

- Accept schema 4, with `icon` validated by shape and not retained, like `hue`. Then decide whether `icon` belongs in the registry fingerprint, which it does not when it is presentation-only, and whether schema 3 is still accepted.
- Or pin the schema 3 document in the adapter until the contract moves, and record that pin in the spec.

Check the File Rollup Format document (`docs/project/architecture/file-rollup-format/file-rollup-format.md` in MetaBrowser) for what else schema 4 changes before choosing.

## Notes

2026-09-14 DECISION (user): accept schema 3 and schema 4. Validate icon by shape and do not retain it, like hue. Presentation fields (icon, hue, deviation, and similar) stay out of the registry fingerprint, so both documents share one classification identity. Verified 2026-09-14: MetaBrowser main's schema-4 registry differs from the last schema-3 one (1e1f5f912) only by icon on every group and family, plus a presentation-only swift hue and deviation change; kinds, extensions, groups and families are identical. Also note MetaBrowser's stale doc line (file-rollup-format.md:214, 'Icons ... remain outside the type registry') on the MetaBrowser side.
