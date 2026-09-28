---
type: is
id: is-01m3mcwvsynn6f2sezxwzndq1n
title: 0.2.0 signed tag v0.2.0 on 6ec77163 with a checked release body
kind: task
status: closed
priority: 0
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m3mcwwdde641b8sphsryhrhm
parent_id: is-01m3mcwtkbvr9kj5j2qwpyyd3j
created_at: 2026-09-28T16:17:18.141Z
updated_at: 2026-09-28T17:12:52.851Z
closed_at: 2026-09-28T17:12:52.836Z
close_reason: Signed v0.2.0 (SSH, ED25519 SHA256:ydo9…) on 6ec77163, verified with a temporary allowed-signers file, pushed; clean clone validated by resolve_plan (release, publish=true, 0.2.0).
resolution: null
duplicate_of: null
---
release_body.py on docs/project/release-notes/0.2.0.md, gh markdown render with zero <br>, git tag -s v0.2.0, git tag -v, push; clone the tag and run resolve_plan --validate-checkout. Needs the maintainer's signing key.
