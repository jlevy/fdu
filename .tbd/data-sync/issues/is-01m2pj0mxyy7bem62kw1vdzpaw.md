---
type: is
id: is-01m2pj0mxyy7bem62kw1vdzpaw
title: crates.io OIDC publish job with digest checks
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2pj0q04xsqtjqj2v62vhhq5
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:32.093Z
updated_at: 2026-09-24T07:50:45.810Z
closed_at: 2026-09-24T07:50:45.809Z
close_reason: "Implemented by PR #123 as the single publish job (crates.io via bootstrap token, then rust-lang/crates-io-auth-action OIDC; digest compared before upload and after). Signed-tag verification stays on fdu-2o7h."
resolution: null
duplicate_of: null
---
crates-io-auth-action inside the publish job only; package from a git checkout, compare both digests, wait between crates.
