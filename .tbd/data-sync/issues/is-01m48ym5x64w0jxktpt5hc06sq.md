---
type: is
id: is-01m48ym5x64w0jxktpt5hc06sq
title: "PR #174 B4: golden-observability test list kept in Makefile and package.json (Makefile:293-295, package.json:19)"
kind: bug
status: closed
priority: 3
version: 4
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m48ykwg1fz7tsw30h6xfhtmd
hold: null
hold_until: null
created_at: 2026-10-06T15:51:56.581Z
updated_at: 2026-10-06T16:32:44.496Z
started_at: 2026-10-06T15:52:01.595Z
closed_at: 2026-10-06T16:32:44.495Z
close_reason: "fixed in 7934f4a6 (local fix-174-B): Make target runs npm run check:golden-observability"
resolution: null
duplicate_of: null
---
Low suggestion. Makefile:293-295 and package.json:19 list the same tests separately. Fix: Make target runs $(NPM) run check:golden-observability, keeping $(NODE_INSTALL_STAMP). PR #174, review B: https://github.com/jlevy/fdu/pull/174#issuecomment-6020029670

## Notes

Fixed in 7934f4a6: make golden-observability runs $(NPM) run check:golden-observability.
