---
type: is
id: is-01m2pj0md1j5nsjfay2prdr5hc
title: release.yml release mode with signed-tag verification
kind: task
status: open
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
  - type: blocks
    target: is-01m2pj0pn53jgvgbty93drg2zh
parent_id: is-01m2pj0jfrvm78rpmv6rwc2ktv
created_at: 2026-09-17T02:09:31.551Z
updated_at: 2026-09-17T02:09:33.852Z
---
Verify the SSH-signed tag against a checked-in allowed-signers file; build and validate in the tag run (wheels are not byte-reproducible, so the rehearsal set cannot be promoted).
