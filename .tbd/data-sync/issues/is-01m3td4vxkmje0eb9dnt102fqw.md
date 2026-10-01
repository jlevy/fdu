---
type: is
id: is-01m3td4vxkmje0eb9dnt102fqw
title: "PR #170 review R5: redaction misses step details, declared values and the user name"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:06.975Z
updated_at: 2026-10-01T00:17:31.040Z
started_at: 2026-10-01T00:17:31.039Z
---
scripts/release/stability_pass.py: verdict() :1325-1329 and note() :1390-1394 print step detail unredacted; regime_lines() :1478-1480 prints declared env values and the login name. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
