---
type: is
id: is-01m3q0d97mjbmn75fpw19ggr4y
title: Decide whether to squash the round's large evidence commits
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T16:36:47.988Z
updated_at: 2026-09-29T16:36:52.201Z
closed_at: 2026-09-29T16:36:52.201Z
close_reason: "Skipped: the round's history packs to 2.0 MB compressed (42.8 MB raw, 198 blobs) against a 19 MB repository pack, under the maintainer's few-MB threshold. History kept intact; no force-push."
resolution: null
duplicate_of: null
---
Measured 2026-09-29: the round adds 198 blobs, 42.8 MB raw, which pack to 2.0 MB compressed (git pack-objects HEAD ^e5a71c8a), against a 19 MB repository pack. The largest blobs are nine versions of the generated timeline.json (~1.2 MB each raw) and the run.json artifacts. Under the maintainer's rule (skip if only a few MB compressed), the squash is skipped and the research-loop history kept.
