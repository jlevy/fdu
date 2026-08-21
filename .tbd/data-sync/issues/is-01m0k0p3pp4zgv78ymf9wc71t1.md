---
type: is
id: is-01m0k0p3pp4zgv78ymf9wc71t1
title: Point SKILL, README, and the spec at --docs, then re-run the gate
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k0nb25qmg8fpvaqybdmpc2
created_at: 2026-08-21T20:37:34.037Z
updated_at: 2026-08-21T20:49:08.320Z
closed_at: 2026-08-21T20:49:08.319Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 115 goldens).
---
Point the other surfaces at the guide so a reader or agent finds it:

- SKILL.md: name `--docs` where it already names `--skill`
- README: the Install/Start Here path should mention it
- the composable CLI spec: record `--docs` as part of the discovery surface, beside
  `--skill` and bare `fdu`, which the spec already treats as deliberate discovery
  behavior

Then `make check`, review the golden diff, and push to PR #37.
