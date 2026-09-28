---
type: is
id: is-01m3kjwsmxyw2qe5kznj3t5j29
title: Ship the native fdu binary in the wheel as the fdu command
kind: feature
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T08:42:52.956Z
updated_at: 2026-09-28T08:42:52.956Z
---
Follow-up to fdu-03yn. The wheel's fdu console script starts a Python interpreter (22 ms floor) before the native CLI; a native binary starts in ~8 ms. Tools like ruff and uv ship the binary in the wheel's scripts data directory. Package the fdu CLI binary alongside the pyo3 extension (maturin data/scripts or a separate bin build) for the five release wheels, keep the Python API unchanged, and update release.yml, inspect_artifacts, and the release tests. Target for 0.2.x, not tonight.
