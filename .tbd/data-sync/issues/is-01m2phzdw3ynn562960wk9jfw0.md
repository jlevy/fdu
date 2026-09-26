---
type: is
id: is-01m2phzdw3ynn562960wk9jfw0
title: Repository security settings before announcing 0.1.0
kind: chore
status: closed
priority: 0
version: 3
labels:
  - release
  - security
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:52.098Z
updated_at: 2026-09-26T01:01:19.703Z
closed_at: 2026-09-26T01:01:19.702Z
close_reason: Enabled and verified private vulnerability reporting, immutable releases, v* tag update/deletion ruleset, secret scanning, and push protection for the public repository.
resolution: null
duplicate_of: null
---
SECURITY.md and the release runbook send reporters to GitHub private vulnerability reporting, which
is disabled on jlevy/fdu. Also unset: a `v*` tag ruleset, immutable releases, secret-scanning push
protection. Needs the maintainer's approval to change repository settings.
