---
type: is
id: is-01kzkvb2y6jhmtzna9e6vh5nxt
title: Make golden scripts and fixture bytes portable on Windows
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies: []
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T18:07:42.533Z
updated_at: 2026-08-09T18:39:23.192Z
closed_at: 2026-08-09T18:39:04.067Z
close_reason: The 25-scenario CLI golden contract is wired into Make and locked audits, its update workflow is proven, and CI run 31329423861 passes it on Linux, macOS, and Windows.
---
The npm scripts quote their glob with POSIX single quotes, which cmd.exe preserves literally, and the size-sensitive fixture lacks an LF checkout attribute. Use portable double-quoted glob arguments, force tests/golden to LF, and validate the matrix on Windows.
