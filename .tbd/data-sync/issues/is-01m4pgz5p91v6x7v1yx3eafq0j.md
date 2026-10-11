---
type: is
id: is-01m4pgz5p91v6x7v1yx3eafq0j
title: "PR #197 A1: finish current-version output expectations"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T22:22:38.792Z
updated_at: 2026-10-11T22:22:53.529Z
started_at: 2026-10-11T22:22:53.528Z
---
High finding in review A https://github.com/jlevy/fdu/pull/197#issuecomment-6114524326. At head 9b696fa2a6e47241069cb07d4b99131b2da71841, tests/golden/cli-surface.tryscript.md:208 and :1037 and tests/parity/deviations-python.diff:1818 and :1819 retain current-output version 0.4.0 while product metadata is 0.5.0. Extend tests/release/test_metadata.py to catch these outputs, update four literals without recording parity on macOS, and clarify docs/project/guides/release-process.md version-bump checklist. Coordinator owns integration gates and CI; no runtime changes.
