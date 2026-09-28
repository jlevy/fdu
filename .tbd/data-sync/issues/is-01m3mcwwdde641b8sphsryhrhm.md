---
type: is
id: is-01m3mcwwdde641b8sphsryhrhm
title: "0.2.0 publish: release.yml on v0.2.0, environment approval, registries verified"
kind: task
status: closed
priority: 0
version: 4
delegate: claude-code
labels: []
dependencies: []
parent_id: is-01m3mcwtkbvr9kj5j2qwpyyd3j
hold: null
hold_until: null
created_at: 2026-09-28T16:17:18.764Z
updated_at: 2026-09-28T18:22:21.528Z
started_at: 2026-09-28T17:12:53.225Z
closed_at: 2026-09-28T18:22:21.518Z
close_reason: "Published and announced: crates.io + PyPI identical, GitHub release 11 assets, docs.rs built both crates, uvx fdu@0.2.0 and @latest print fdu 0.2.0; make release-announced exit 0."
resolution: null
duplicate_of: null
---
Dispatch with publish=true on the tag, confirm event/ref/SHA, maintainer approves the release environment, watch to the end, confirm crates.io and PyPI list 0.2.0 with the manifest digests; then the GitHub release and announcement.

## Notes

2026-09-28: publish run 36456542352 on v0.2.0 (6ec77163) approved via the pending-deployments API at the user's direction; Publish job success. crates.io fdu-core/fdu 0.2.0 and PyPI fdu 0.2.0 live; registry audit identical on all three. GitHub release https://github.com/jlevy/fdu/releases/tag/v0.2.0 with 11 assets, body identical to the tag's notes. uvx fdu@0.2.0 and fdu@latest print 'fdu 0.2.0'. Pinned branch release/v0.2.0 deleted. Open: docs.rs builds for both crates were pending minutes after publish; rerun make release-announced.
