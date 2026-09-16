---
type: is
id: is-01m2nh4zx5f0ty8zasxxr25qn0
title: Write a field-level reference for the fdu.report/5, fdu.report/6, fdu.cache/1, and fdu.stream/1 machine schemas
kind: task
status: open
priority: 2
version: 1
labels:
  - stack-followup
  - docs
dependencies: []
created_at: 2026-09-16T16:35:11.395Z
updated_at: 2026-09-16T16:35:11.395Z
---
No live document states fdu's machine-output envelopes field by field, at `16efcd0`.

The four schemas are `fdu.report/5` (`REPORT_SCHEMA`, crates/fdu-core/src/report_format.rs:61),
`fdu.report/6` (`CONTENT_REPORT_SCHEMA`, :63), `fdu.cache/1` (`CACHE_SCHEMA`, :70), and
`fdu.stream/1` (`STREAM_SCHEMA`, :1480). They moved three times in one night (PRs #63, #65, #67),
and a consumer today infers their fields from SKILL.md's validation checklist
(crates/fdu/src/skills/SKILL.md:205-224), the `--docs` one-liners, README prose, and the goldens.

Fields a reference must state, with where each is rendered:

- The report envelope's `ignore_rules` object (`limits.budget`, `limits.line_limit`, `applied`,
  `refused`, `refusals[]`), or null when no `.gitignore` was read:
  report_format.rs:596 and `ignore_rules_json` at :609.
- The per-row `ignored`: an object on tree, summary, and extension rows (`ignored_json`, :852),
  `true`/`false` on file rows, and absent or null when observation was off.
- `analysis`, present only under `fdu.report/6` (:597-599), and which views select `/6`
  (`report_schema`, :1323-1331).
- The cache-status rows: `path`, `bytes`, `content_bytes`, `state`, plus `root`/`entries` for a
  current snapshot, `stale_reason`/`format_version` for a stale one, and `leftover_kind` for a
  leftover (`render_cache_status`, :1600-1635).
- The stream's `change` record: `op` (`upsert`, `remove`, `invalidate`), `path`, `clock`, and
  `kind`/`bytes`/`allocated`/`mtime_ns`/`ignored` when present (`render_change`, :1498-1540).

The audit (doc-drift-audit at 16efcd0, "Documents that should exist" item 1) proposed
`docs/project/guides/machine-output-schemas.md` or a section in the surface architecture.
The docs PR for root, architecture, and guides adds the owner section to
docs/project/architecture/fdu-surface-architecture.md ("Machine Output Schemas"): names, what
selects each, and the constant that defines it, but no field reference. This bead is the field
reference, which that section and docs/project/guides/release-process.md should then link.
Keep the reference next to the golden that pins each shape, or generate it, so it cannot drift
the way the composable-CLI plan's "Schemas and Compatibility" section did (it still says
`fdu.report/1`).
