---
type: is
id: is-01kzypf1yd2v4g8q8tk2v1xmxs
title: Implement or explicitly defer content analysis in watch mode
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:14:08.460Z
updated_at: 2026-08-14T00:04:20.421Z
---
The CLI now explicitly rejects enabled content analysis with --watch, the Python watch feed remains metadata-only, and user-facing docs call content analysis one-shot. Implement incremental content reanalysis on metadata deltas before claiming full mode composability.
