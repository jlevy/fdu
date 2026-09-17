---
type: is
id: is-01m2pj0ngk9sgeppchy5zd6z54
title: "registry_state: classify partial uploads separately from conflicts"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2pj0p31w8pg5wahbayvvy8d
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:32.690Z
updated_at: 2026-09-17T02:09:33.280Z
---
A subset of files with matching hashes (partial upload or PyPI JSON API lag) is currently a conflict (registry_state.py:63-82).
