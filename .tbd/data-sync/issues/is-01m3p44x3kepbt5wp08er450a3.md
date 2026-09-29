---
type: is
id: is-01m3p44x3kepbt5wp08er450a3
title: "H176: the one-shot tree tier maintains exactly the reducers the requested views read (no by_ext on a Tree-only request)"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T08:22:53.299Z
updated_at: 2026-09-29T08:22:53.299Z
---
From the Fable plan review (F6a). Under the H172 tier (RetainedState::Tree), intern no ext_id and build or merge no by_ext map (index.rs:1690-1691, 5259-5270, 1713-1715, 1740), because only Types/Families read by_ext (query_report.rs:1933, 2041-2062) and those views make the tier ineligible. ~-170M of 257M consumer instructions on linux-v6.12 --no-controls; allocations per file 7 -> <=2. Measured inside the H172 cell (Q4). Placebo: aggregate-summary; fdu PATH --view types byte-identical.
