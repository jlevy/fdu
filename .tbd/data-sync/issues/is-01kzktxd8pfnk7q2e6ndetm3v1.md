---
type: is
id: is-01kzktxd8pfnk7q2e6ndetm3v1
title: Report tryscript update misassignment for duplicate blocks
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies: []
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T18:00:14.357Z
updated_at: 2026-08-09T18:02:15.184Z
closed_at: 2026-08-09T18:02:15.183Z
close_reason: Filed https://github.com/jlevy/tryscript/issues/47 with a minimal stateful reproduction and updater source-level cause. Kept repeated fdu invocations textually distinct so --update cannot confuse blocks, restored expectations from the sequential run, and documented the workaround.
---
File an upstream issue showing that --update uses indexOf(rawContent) while iterating in reverse, so identical command blocks receive another invocation's captured output in stateful sessions. Restore and review the fdu cache transcript manually.
