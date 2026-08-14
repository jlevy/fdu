---
type: is
id: is-01m01293x5gaacv3vxjdtrg146
title: Polish fdu 0.1.0 packaging and Python API
kind: epic
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01m0129rtmjnmb8y1a4zpwrckp
  - is-01m0129s2s8ctztpscxkjz6vnt
  - is-01m0129sar39fdss9a969ak156
  - is-01m0129sjsp6xht1ae8aebvx0r
  - is-01m0129stsv3wn03hy6m3j4r4t
  - is-01m0129t2wsdsv20mt3bq7s0zh
created_at: 2026-08-14T21:19:05.636Z
updated_at: 2026-08-14T21:19:41.861Z
---
Close the release-audit gaps between the working Rust engine and supportable crates.io, PyPI, uvx, and typed Python artifacts. This epic owns the public Python package contract, artifact identity and contents, portable wheel matrix, artifact-first automation, and installed-consumer acceptance gates. The existing fdu-9cf0 bead remains the final publication action and depends on this epic.
