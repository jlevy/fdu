---
type: is
id: is-01m2pj0rjcedzrm53gsh4jawny
title: Decide free-threaded CPython support
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
  - python
dependencies: []
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:35.820Z
updated_at: 2026-09-17T02:09:35.820Z
---
abi3 wheels cannot load on 3.14t; uv selects a managed 3.14t by default where installed and then builds the sdist. Options: cp314t wheels (thread-safety review of the binding) or a documented --python requirement.
