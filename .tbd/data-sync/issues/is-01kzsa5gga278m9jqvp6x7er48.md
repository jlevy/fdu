---
type: is
id: is-01kzsa5gga278m9jqvp6x7er48
title: "PR#6 D7: adaptive cache/journal cost model is not calibrated enough to be a decision function"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:03:00.617Z
updated_at: 2026-08-11T21:03:00.617Z
---
Previous scan cost untagged by warm/cold regime; maxvnodes coincidence is one host; estimated_journal omits post-replay stat work; G9 counts dirs not fanout. Treat as measured heuristic with hysteresis and fallback. Medium.
