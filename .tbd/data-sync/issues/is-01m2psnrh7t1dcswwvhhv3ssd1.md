---
type: is
id: is-01m2psnrh7t1dcswwvhhv3ssd1
title: "PR #78 review D4: Entry tier identity needs a ScanScope split the plan does not name"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2psnq245vvkwkfs8zt3nfk0
created_at: 2026-09-17T04:23:23.942Z
updated_at: 2026-09-17T04:33:04.608Z
closed_at: 2026-09-17T04:33:04.608Z
close_reason: "Fixed in PR #78 commits 3fb805b7 and e52383d4; disposition posted on the PR"
resolution: null
duplicate_of: null
---
PR #78 delta review (https://github.com/jlevy/fdu/pull/78#issuecomment-5708468480), finding D4.

Name EntryScope without ignore_rules/type_rules/reducers fingerprints; list ScanConfig::scope and observes_controls consumers; test equal entry identities across controls on/off.
