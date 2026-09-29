---
type: is
id: is-01m3p44x3kepbt5wp08er450a3
title: "H176: the one-shot tree tier maintains exactly the reducers the requested views read (no by_ext on a Tree-only request)"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T08:22:53.299Z
updated_at: 2026-09-29T10:40:08.921Z
closed_at: 2026-09-29T10:40:08.921Z
close_reason: "Accepted and merged (0228ea42): exp-180 linux-v6.12 default-tree -13.48%, exp-181 node-modules-dense -10.30%, balanced screen -3.20% wall and peak RSS -79%; placebos at zero; answers identical (171 + 111 comparisons)."
resolution: null
duplicate_of: null
---
From the Fable plan review (F6a). Under the H172 tier (RetainedState::Tree), intern no ext_id and build or merge no by_ext map (index.rs:1690-1691, 5259-5270, 1713-1715, 1740), because only Types/Families read by_ext (query_report.rs:1933, 2041-2062) and those views make the tier ineligible. ~-170M of 257M consumer instructions on linux-v6.12 --no-controls; allocations per file 7 -> <=2. Measured inside the H172 cell (Q4). Placebo: aggregate-summary; fdu PATH --view types byte-identical.
