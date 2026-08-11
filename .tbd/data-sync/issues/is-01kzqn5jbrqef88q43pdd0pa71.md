---
type: is
id: is-01kzqn5jbrqef88q43pdd0pa71
title: "P3: Python Index.watch() iterator"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:47.991Z
updated_at: 2026-08-11T17:18:23.707Z
closed_at: 2026-08-11T17:18:23.706Z
close_reason: Python Index.watch() with GIL released across the wait, empty-batch ticks so the interpreter can always exit, close()/context-manager shutdown, and unsendable declared at the boundary; fdu-py enables watch while the fdu crate still builds cli-only. Benchmark job vocabulary extended with cli-summary, cli-files, watch-stream across schema.py, the JSON schema, and the CLI-job set. docs/project/guides/fdu-design-principles.md records the eleven principles as implemented, with both amendments and the testing hazards; AGENTS.md points at it. make check passes.
---
fdu-py: Index.watch(views=[...], interval=2.0, **selection) returning an iterator of batches, mirroring the CLI stream records and honoring the same Selection. Must release the GIL while blocking on the watcher and support deterministic shutdown (close/context-manager plus interpreter exit) without hanging a Python process - the existing watch shutdown ordering hazard (drop the watcher before joining the worker) applies here too. Note the wheel currently proves the watch layer is deletable: enabling the watch feature for fdu-py is a deliberate packaging change, so confirm the feature matrix and the --no-default-features build stay honest, and update the installed-wheel smoke accordingly.
