---
type: is
id: is-01kzypz62h7n4k9acw4s304fd5
title: Render unavailable metric shares and incomplete content coverage explicitly
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:22:56.976Z
updated_at: 2026-08-14T00:03:56.031Z
closed_at: 2026-08-14T00:03:56.031Z
close_reason: Implemented in c2b646c; full make check and all 16 required PR checks pass.
---
A valid language analysis with only unsupported languages renders 0.0%, even though the exact share is 0/0, and human rows omit preserved coverage. Render unavailable share distinctly and summarize non-analyzed coverage.
