---
type: is
id: is-01m2phzm7r5vw97jr9y4mtzxxs
title: Publish fdu-core, then fdu, 0.1.0 to crates.io
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
    target: is-01m2phzmjh2sk8fzqya61v47rd
parent_id: is-01kzg4c6vnh98mqrpkzw7ydne0
created_at: 2026-09-17T02:08:58.615Z
updated_at: 2026-09-26T17:25:48.496Z
closed_at: 2026-09-26T17:25:48.495Z
close_reason: fdu-core and fdu 0.1.0 published in order; release workflow registry audit found identical artifacts; one-time token revoked; cargo install smoke passed.
resolution: null
duplicate_of: null
---
Short-lived scoped token; reproduce each .crate and compare with the rehearsal digest before upload;
wait for the index to carry fdu-core before publishing fdu; registry audit `--require-identical`;
`cargo install --locked fdu` prints `fdu 0.1.0`; revoke the token.
