---
type: is
id: is-01kzyqdc2hvymv0b082k9g65yc
title: Pin content-basic Unicode whitespace semantics independently of the Rust compiler
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:30:41.870Z
updated_at: 2026-08-14T00:03:56.038Z
closed_at: 2026-08-14T00:03:56.038Z
close_reason: Implemented in c2b646c; full make check and all 16 required PR checks pass.
---
content-basic-v1 currently delegates to char::is_whitespace, whose Unicode table can change across compiler versions without an analyzer version bump. Use the explicit Unicode White_Space set and lock it with tests.
