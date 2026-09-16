---
type: is
id: is-01m2ngg0kmwnat7p7yhkcwh2ww
title: "PR #64 review RN64-2: extension rows' ignored object has no dirs key, and Python ExtensionRow.ignored is an ExtensionTally"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2ngfd0y2yzwg2v10p2j601z
hold: null
hold_until: null
created_at: 2026-09-16T16:23:43.987Z
updated_at: 2026-09-16T16:25:03.130Z
started_at: 2026-09-16T16:25:03.129Z
---
CHANGELOG.md:131-133 and :172-173@d303dc1 list (files, dirs, bytes, allocated) for summary, tree and extension rows and imply IgnoredTally in Python. crates/fdu-core/src/report_format.rs:864-875 ignored_files_json emits {files, bytes, allocated}; YAML passes with_dirs=false; crates/fdu-py/python/fdu/_models.py:542 types ExtensionRow.ignored as ExtensionTally.
