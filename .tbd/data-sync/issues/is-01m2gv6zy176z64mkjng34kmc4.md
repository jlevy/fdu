---
type: is
id: is-01m2gv6zy176z64mkjng34kmc4
title: CLI shows gitignored share in summaries and tree rows, with --exclude-ignored and --only-ignored
kind: feature
status: open
priority: 1
version: 2
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-14T20:54:50.553Z
updated_at: 2026-09-15T05:15:30.428Z
---
DECISION (user, 2026-09-14): once reports observe .gitignore by default, the CLI shows how much of each size is gitignored, for example '1.2 GB (340 MB ignored)', in summary and tree rows, and adds --exclude-ignored and --only-ignored filters. JSON output carries the ignored/unignored split. Human output changes, so goldens and the Python parity corpus change too; read every diff. With --no-gitignore the split is omitted and never shown as zero. Depends on the default-on bead.
