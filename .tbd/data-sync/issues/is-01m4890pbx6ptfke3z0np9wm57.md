---
type: is
id: is-01m4890pbx6ptfke3z0np9wm57
title: "PR #177 C3: correctness scripts derive must-serve pairs from the engine's own request.analyze (cross_warm.py"
kind: task
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T09:34:17.980Z
updated_at: 2026-10-06T15:37:11.086Z
---
Review C (https://github.com/jlevy/fdu/pull/177#issuecomment-6013447138), Low. correctness scripts derive must-serve pairs from the engine's own request.analyze (cross_warm.py:152, warm_cold.py:156); the stability gate checks caught == held with no expected count, so a regression could shrink coverage silently. Add an expected analyzer set per request (SET-MISMATCH) or an expected held count.
