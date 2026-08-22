---
type: is
id: is-01m0nbkjftj3mzm51f91t2nwdb
title: Watch records have no renderer
kind: feature
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T18:26:54.073Z
updated_at: 2026-08-22T18:26:54.073Z
---
Split out of fdu-1kw3, which closed the cache-status half.

Index.watch yields Change values and nothing turns them into the CLI's bytes, so the parity shim prints repr() and two sessions differ: 'Build a Tree and Capture a Watch Session' and 'Text Repaints Are Separated From One Another'.

The cache-status fix is the template: the human layout belongs in report_format beside the others, the CLI delegates to it, and the binding exposes it. The watch case is harder because the CLI's watch loop also owns repaint timing and the jsonl stream envelope, so decide what is a renderer and what is loop behaviour before moving anything.
