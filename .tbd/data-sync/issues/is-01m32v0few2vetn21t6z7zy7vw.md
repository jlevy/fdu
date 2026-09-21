---
type: is
id: is-01m32v0few2vetn21t6z7zy7vw
title: "PR #103 review R2: size/time predicates without --kind cover whole subtrees, undocumented (High)"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6ewht09ryvnt0f5zmhz5
created_at: 2026-09-21T20:37:36.859Z
updated_at: 2026-09-21T20:37:36.859Z
---
R2 High. query_report.rs walk() ~1327-1356. --modified-since in the default view changed meaning. Fix (pick one): document the coverage rule for time and size predicates, update README example, add a golden for a no-kind time filter; or decide time predicates should not cover and change the engine.
