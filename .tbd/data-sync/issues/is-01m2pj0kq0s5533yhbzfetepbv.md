---
type: is
id: is-01m2pj0kq0s5533yhbzfetepbv
title: "Trusted publishers: PyPI fdu, crates.io fdu-core and fdu"
kind: chore
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2pj0mxyy7bem62kw1vdzpaw
  - type: blocks
    target: is-01m2pj0p31w8pg5wahbayvvy8d
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:30.847Z
updated_at: 2026-09-27T08:24:03.476Z
closed_at: 2026-09-27T08:24:03.474Z
close_reason: "Publisher setup completed: fdu-o5st records both crates.io bindings to jlevy/fdu, release.yml, protected release environment, bootstrap secret deletion and token revocation. PyPI OIDC publication succeeded in release run 36219577994. User independently confirmed temporary-token revocation."
resolution: null
duplicate_of: null
---
Register only after the protected environment exists and the crates are published.
