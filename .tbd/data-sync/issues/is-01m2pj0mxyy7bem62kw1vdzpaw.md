---
type: is
id: is-01m2pj0mxyy7bem62kw1vdzpaw
title: crates.io OIDC publish job with digest checks
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2pj0q04xsqtjqj2v62vhhq5
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:32.093Z
updated_at: 2026-09-17T02:09:34.211Z
---
crates-io-auth-action inside the publish job only; package from a git checkout, compare both digests, wait between crates.
