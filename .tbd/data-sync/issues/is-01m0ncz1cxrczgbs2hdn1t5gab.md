---
type: is
id: is-01m0ncz1cxrczgbs2hdn1t5gab
title: Write the experiment loop skill
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
child_order_hints:
  - is-01m0nkj4vderecrkj9ftkhy38c
  - is-01m0nkj59jhwg19680g3b4zb6s
  - is-01m0nkj5q8hb5wbqj0ce2prnfx
created_at: 2026-08-22T18:50:38.363Z
updated_at: 2026-08-22T20:53:04.621Z
closed_at: 2026-08-22T20:53:04.620Z
close_reason: "Skill implemented at ~/.claude/skills/experiment-loop (SKILL.md + references/{contract,statistics,traps,worked-examples}.md). Written as principles + adaptable pieces per design: invariant core, adoption knobs, four result shapes, starter schemas, failure catalog, four domain mappings, pointers to softschema CLI and both exemplar repos. Whether it later ships beside softschema remains the spec's open question."
---
The skill is the transferable core — metabrowser proved the method transfers with zero shared code, so the skill carries: the invariant core (12 agreements both loops converged on independently, see spec), the adoption ladder (rungs 0-3), the knobs and how to choose them (statistics by tier, schema origin by field count, drift gate by record size, measurement mode by dependency policy), the four comparison shapes, metric roles, decisions incl. abandoned-with-budget, and every named trap from both records: two hypotheses competing for one cost, the harness in its own profile, a counter reading zero, a 0x0 viewport, a hidden pane's idle callbacks, predicting a component while scored on a wall, a fresh server is not a cold scan, a plausible number that took three measurements to kill.
