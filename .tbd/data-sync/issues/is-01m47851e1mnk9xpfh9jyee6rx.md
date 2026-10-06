---
type: is
id: is-01m47851e1mnk9xpfh9jyee6rx
title: Demo scripts and cli-animate README use the one-flag command
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-05-fdu-view-analyze-redesign.md
labels: []
dependencies: []
parent_id: is-01m47831pcsbtvygctgzqfvzc8
created_at: 2026-10-05T23:59:57.376Z
updated_at: 2026-10-05T23:59:57.376Z
---
packages/cli-animate/examples/fdu/linux.yaml second step: fdu linux --view code,documents --limit 6 (the code table), or fdu linux --analyze code --view languages,documents --limit 6 if the maintainer prefers the compact per-language rows (spec open question 1). showcase.yaml and views.yaml: 'fdu cpython --analyze code --view languages --limit N' stays valid as a control form; switch to '--view code --limit N' where the code table is wanted. Update packages/cli-animate/README.md and the cli-animate plan spec's command listings. Regeneration of the recording itself stays with fdu-qci5.
