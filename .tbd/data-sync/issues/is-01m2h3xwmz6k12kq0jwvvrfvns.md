---
type: is
id: is-01m2h3xwmz6k12kq0jwvvrfvns
title: "PR #60 review PR60-DOCS-3: streaming-parity plan still says a binary may be compiled with gitignore support"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3xgn1f3azaqbcvh9mzxas
created_at: 2026-09-14T23:27:09.470Z
updated_at: 2026-09-15T00:19:14.155Z
closed_at: 2026-09-15T00:19:14.154Z
close_reason: "3a4d384 (PR #60): the streaming-parity plan's rule names only the watch build feature."
resolution: null
duplicate_of: null
---
PR #60 at df43bdd: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md:209-210 states in the present tense that the default CLI does not select a streaming mode because the binary was compiled with watch or gitignore support; gitignore is no longer a build shape.
