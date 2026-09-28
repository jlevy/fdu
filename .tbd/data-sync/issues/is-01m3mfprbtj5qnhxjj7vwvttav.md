---
type: is
id: is-01m3mfprbtj5qnhxjj7vwvttav
title: Upgrade the Node 20 artifact actions in release.yml
kind: chore
status: open
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:06:23.737Z
updated_at: 2026-09-28T22:51:07.887Z
---
The 0.2.0 rehearsal (run 36450043255) and the 0.1.0 publish (run 36219577994) both log: 'Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/download-artifact@d3f86a10..., actions/upload-artifact@ea165f8d...' (release.yml pins upload-artifact v4.6.2 and download-artifact v4.3.0). When GitHub removes the forced-run shim the release workflow breaks, and a tag cannot pick up a workflow fix without a new commit and version. Upgrade to current SHA-pinned releases that clear the 14-day cool-off (SUPPLY-CHAIN-SECURITY.md), rerun a rehearsal, and check the evidence and publish jobs still download exactly the eight files (the artifact action major versions changed merge/pattern behavior before). Found in fdu-n2hc.

## Notes

2026-09-28: implemented on claude/fdu-alternatives-research-qx0xn0 (PR #155), merged at 2d1eea9d/e0923063. 16cbe206: upload-artifact v7.0.1, download-artifact v8.0.1 (SHA-pinned, >14 days). Pending: independent review and CI, then close.
