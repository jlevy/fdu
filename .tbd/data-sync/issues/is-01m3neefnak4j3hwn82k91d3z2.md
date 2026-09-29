---
type: is
id: is-01m3neefnak4j3hwn82k91d3z2
title: run_installed_cli_qa.py reads GNU time's exit status as wall time after a non-zero exit
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T02:03:38.537Z
updated_at: 2026-09-29T02:03:38.537Z
---
After 'Command exited with non-zero status 2', the harness parses 2 as the wall time (documents shows 2.000 s for a 0.02 s run). Parse the elapsed field by label. Found in the 0.2.1 stability pass.
