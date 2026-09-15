---
type: is
id: is-01m2jzbn8fn9ek39ebfp0wf1yz
title: "PR #61 review PR61-GUIDE-3: literal-reading traps in the by-hand publish section"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2jzadk7w8m1xcsewzwg5wj1
created_at: 2026-09-15T16:45:46.637Z
updated_at: 2026-09-15T17:10:18.257Z
closed_at: 2026-09-15T17:10:18.254Z
close_reason: "951c5a0, 7da5d76: bash/zsh named; tag step in a clean clone detached at the release commit on main; gh release create chained on registry_state.py --require-identical (exit 3 unless identical, unit-tested); bounded retry for post-publish missing"
resolution: null
duplicate_of: null
---
PR #61 at eb89150, docs/project/guides/release-process.md:129,172,208,264,297-299.

- `read -rs` is not POSIX, but the section promises a POSIX shell.
- The tag step's fetch and tag commands name no checkout: say a clean `main` at the
  release commit.
- `gh release create` runs even when `registry_state.py` exits 2: chain them with `&&`.
- A `missing` right after `cargo publish` can be download-endpoint lag: give a bounded
  retry.
