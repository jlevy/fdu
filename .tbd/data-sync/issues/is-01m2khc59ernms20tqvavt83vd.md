---
type: is
id: is-01m2khc59ernms20tqvavt83vd
title: Annotate types, families, languages, and documents rows with their ignored share
kind: feature
status: open
priority: 2
version: 2
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-15T22:00:37.421Z
updated_at: 2026-09-16T00:18:22.125Z
---
Follow-up to fdu-5ryb, decided as Q8 (recorded on fdu-elnn and fdu-5ryb, PR B design): PR B (branch claude/gitignore-default-on) annotates summary, tree, and extension rows with their ignored share, and leaves the grouped metric views unannotated.

State at ea671de: `MetricRow` (crates/fdu-core/src/query/query_report.rs) has no `ignored` field. `--exclude-ignored` and `--only-ignored` already filter these views, because `metric_summary` reads the walked rows, but an unfiltered `--view types` shows no share, which is uneven with `--view extensions` under 'a roll-up partitions what it reports on'.

Direction: add `ignored: Option<IgnoredTally>` (files, bytes, allocated) to `MetricRow` and its total, counted from `FileRow::ignored` in `metric_summary`; serialize under the current `fdu.report/6` if unreleased, else bump; text appends ' (N ignored)' after the detail as the other rows do; Python `MetricRow.ignored`; goldens and parity.

Acceptance: grouped rows sum their ignored shares to the summary's; `null` under `--no-gitignore`; goldens re-recorded and read.

## Notes

2026-09-15, review of PR #65, on Q5 (suffix placement). The placement ships unchanged: one suffix after the detail on every row, which keeps the tree's size, bar and percentage columns aligned, renders every row through one function, and moved the goldens once.

The reviewer's one wrinkle is worth folding into this bead rather than a golden churn of its own: a tree row now carries two adjacent parentheticals, `. (7 files) (128 B ignored)`. If these rows are touched for the grouped views, fold the share into the existing parenthetical instead, `. (7 files, 128 B ignored)`, which is still after the detail and still one placement.

Rejected: moving the suffix beside the size on summary rows only. That is two placements for one concept, for a one-row view where either reading of `1.2 GB (340 MB ignored)` is clear.
