---
type: is
id: is-01kzkt8wd2cah0n9kp4h79t8y0
title: Report tryscript multi-command console block hazard
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies: []
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:49:01.729Z
updated_at: 2026-08-09T17:50:06.241Z
closed_at: 2026-08-09T17:50:06.240Z
close_reason: Filed https://github.com/jlevy/tryscript/issues/46 with a minimal multi-prompt reproduction, false-pass risk analysis, and proposed parse-error regression; documented one command per console fence in the fdu spec.
---
File an upstream tryscript issue with a minimal reproduction showing that multiple dollar prompts in one console fence are concatenated as one shell command. Recommend a parse error unless sequential-command semantics are implemented.
