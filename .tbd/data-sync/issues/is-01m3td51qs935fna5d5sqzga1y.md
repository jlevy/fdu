---
type: is
id: is-01m3td51qs935fna5d5sqzga1y
title: "PR #170 review R9: with --wheels, only the version string is checked"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:12.935Z
updated_at: 2026-10-01T03:26:33.119Z
started_at: 2026-10-01T00:17:34.847Z
closed_at: 2026-10-01T03:26:33.118Z
close_reason: "Fixed in df829ec8 and 9e53c996: --no-build; a given wheel needs the rehearsal record, tagged or not. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/release/stability_pass.py:803-805, :605-613 accept any bare 'fdu X.Y.Z' with --wheels; the rehearsal files include the sdist, which uv tool install --no-index --find-links would build on a platform with no wheel. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
