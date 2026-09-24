---
type: is
id: is-01m395z2xdn7yqec3ktz7bav74
title: Run release.yml gate scripts on the uv-pinned interpreter, not the runner's python3
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-24T07:44:32.162Z
updated_at: 2026-09-24T07:44:32.162Z
---
release-engineering-rules: give release-support code an interpreter the project pins. The plan, crate, evidence, release-environment, and publish jobs call scripts/release/*.py with the image's python3; setup-uv is already present, so run them as 'uv run --no-project --python 3.12 python' the way make release-test does. Deferred from PR #123 review R5 (Low, inherited from the pre-existing jobs).
