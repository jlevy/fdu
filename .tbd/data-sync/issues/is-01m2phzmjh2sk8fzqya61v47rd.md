---
type: is
id: is-01m2phzmjh2sk8fzqya61v47rd
title: Publish fdu 0.1.0 to PyPI with uv publish
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
  - packaging
dependencies:
  - type: blocks
    target: is-01m2phzmxbwajz8eqrcv6rzfks
parent_id: is-01kzg4c6vnh98mqrpkzw7ydne0
created_at: 2026-09-17T02:08:58.961Z
updated_at: 2026-09-26T17:25:48.786Z
closed_at: 2026-09-26T17:25:48.785Z
close_reason: fdu 0.1.0 source distribution and five wheels published to PyPI by the protected GitHub trusted publisher; registry audit found identical files and hashes; uv wheel install and execution passed.
resolution: null
duplicate_of: null
---
Upload the validated sdist and five wheels with a short-lived account token; registry audit identical;
`uv tool run --no-config --no-build --python 3.12 --from fdu==0.1.0 fdu --version` and again with a GIL
3.14; delete the token.
