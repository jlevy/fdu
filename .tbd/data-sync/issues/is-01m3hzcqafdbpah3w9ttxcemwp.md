---
type: is
id: is-01m3hzcqafdbpah3w9ttxcemwp
title: Include Code in CLI view help vocabulary
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m3hz327rqpdxwdh0f5e0hyj5
created_at: 2026-09-27T17:42:48.910Z
updated_at: 2026-09-27T17:42:48.910Z
---
Full stack review S2 (Low), owning PR133. crates/fdu/src/cli.rs:549-551 advertises supported views but omits code. Parser, docs and code golden support it. Add code to help list and update/read shared help golden through normal workflow. Review only; no source fix yet.
