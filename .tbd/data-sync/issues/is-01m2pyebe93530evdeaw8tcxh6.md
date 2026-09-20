---
type: is
id: is-01m2pyebe93530evdeaw8tcxh6
title: "P2.2.7: Text over the answer model; Report carries status, provenance, and the request echo"
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
    target: is-01m2pyebrt6j2ghh4c4vcdvyxg
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
hold: null
hold_until: null
created_at: 2026-09-17T05:46:44.040Z
updated_at: 2026-09-20T05:13:36.244Z
started_at: 2026-09-20T05:13:36.242Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 7. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `report_format.rs`: `render_text_metrics` (`:231-311`) and `share_metric_note` (`:319-325`) decide what to show from unit presence and `pages()`; keep `render` (`:106-113`) and add `write(report, format, color, out: &mut dyn io::Write)` for streaming.
- `query/query_report.rs`: `Report` (`:779-829`) carries `status`, `provenance`, and the request echo; `notes` and `ignored_entries` stay text-only and the schema marks them off the wire.
- Call sites: `crates/fdu/src/cli.rs:692`, `:707`, `:729`, `:746-751`, `:812`, `:847`, `:849`, `:956-978`, `:1095-1106`, `:1159-1166`, and tests `:2514-2517`, `:2796-2799`; `execution.rs:624-625`; `examples/perf_probe.rs:785`.

**Tests**

- `crates/fdu/tests/cli_color.rs:45` and `:118`; text goldens change only where unit presence or `pages()` changes what text shows.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
