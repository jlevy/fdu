---
type: is
id: is-01m2kfrqb05nnfapw608x1wbrp
title: "PR #63 review PR63-CLI-1: --gitignore-budget reads as live but is inert until PR B"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:31.963Z
updated_at: 2026-09-17T02:10:33.753Z
closed_at: 2026-09-17T02:10:33.752Z
close_reason: "#65 merged; --gitignore-budget is live"
resolution: null
duplicate_of: null
---
PR #63 review PR63-CLI-1 at 1fd71a9: cli.rs:365-367,589. Annotate the help (applies when .gitignore is read) rather than hiding, since PR B flips the default in the same release; check the flags join cache scope consistently once controls are observed.
