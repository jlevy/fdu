---
type: is
id: is-01m3nvrp6dnwewgsx2s50j5bkj
title: "Release: create the GitHub release inside Actions (flowmark-rs model) so no step downloads artifacts locally"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-29T05:56:24.397Z
updated_at: 2026-09-29T05:56:24.397Z
---
Steps 4 and 8-9 of the Release Checklist download Actions artifacts to the maintainer's machine (make release-candidate, make release-published, gh release create with 11 assets). Cloud agent containers cannot reach productionresultssa*.blob.core.windows.net, so 0.2.1's steps 8-9 needed the maintainer. flowmark-rs (.github/workflows/release.yml lines 174-187) instead has a job with contents: write that downloads the artifacts inside Actions and publishes the release with softprops/action-gh-release. Design an equivalent announce job for fdu: runs after publish succeeds, re-verifies the registries against the manifest (registry_state), attaches the 11 files and notes.md-derived body; keep publication authority confined as tests/release/test_metadata.py requires (likely a job gated by the release environment). Target 0.2.2.
