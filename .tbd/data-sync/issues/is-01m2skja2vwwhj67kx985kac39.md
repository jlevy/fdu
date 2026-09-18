---
type: is
id: is-01m2skja2vwwhj67kx985kac39
title: "PR #86: README watch example hangs python-check"
kind: bug
status: in_progress
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m2sgadh8f84z9wxtrhhzszk9
created_at: 2026-09-18T06:34:22.670Z
updated_at: 2026-09-18T06:34:28.783Z
---
The first-user landing page added a live index.watch() loop. test_readme_examples.py execs every Python fence, so make python-check never returns. Split the watch loop into its own fence, compile it, and skip exec.
