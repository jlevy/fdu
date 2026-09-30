---
type: is
id: is-01m3qrhfc55cdy67aaetregzck
title: Profile opened discovery's post-walk drain and validation (+1.2% wall after the automount fix)
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
created_at: 2026-09-29T23:38:31.173Z
updated_at: 2026-09-29T23:38:31.173Z
---
exp-196 (H184, the fdu-d2fn non-regression screen) measured opened-discovery wall +1.19% [+0.56%, +3.51%] on linux-v6.12 after every listing route moved to the native reader, while the discovery component itself was +0.06% [-0.38%, +1.70%]. So the cost sits in what follows the walk (the drain and validation), not the listing. Profile the opened route at 45943211 against 4bc9b738 and either explain the difference or remove it. Note: the shared linux-v6.12 fingerprint (perf/results/tree-linux-v6.12.json) drifted at 19:00 UTC on 2026-09-29 (.git/index touched); exp-196 used a re-registered fingerprint (tree-linux-v6.12-d2fn.json).
