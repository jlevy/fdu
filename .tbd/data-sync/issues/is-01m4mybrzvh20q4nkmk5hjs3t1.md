---
type: is
id: is-01m4mybrzvh20q4nkmk5hjs3t1
title: Color ages and sizes by magnitude, after kash's files listing
kind: feature
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
child_order_hints:
  - is-01m4p9yaayv0nn574mwa4age37
  - is-01m4pfsmyd25m58ce70tbe59gt
hold: null
hold_until: null
created_at: 2026-10-11T07:38:14.393Z
updated_at: 2026-10-11T22:02:09.228Z
started_at: 2026-10-11T07:38:33.586Z
---
Maintainer decision 2026-10-11: color human ages and sizes by magnitude, following kash's files command (src/kash/config/text_styles.py color_for_qty), adapted to fdu's scale. Ages, by duration on fdu's own unit ladder: under 1m bold bright red, under 1h bold yellow, under 1d bold bright green, under 1 display month bold cyan, under 1 display year blue, older gray (kash's weeks tier becomes months; kash has no month tier). Sizes: under 16 KiB gray (zero included); 16 KiB to 1 MiB blue; 1 MiB to 1 GiB plain; 1 GiB to 100 GiB bold bright green; 100 GiB and up bold yellow. Secondary (gray parenthetical) quantities keep their gray, bold at 1 GiB+. --long and flat output stay plain. Update report_format.rs module doc, fdu-output-design.md, unit/terminal tests and goldens. Stacks above #194 for 0.5.0.
