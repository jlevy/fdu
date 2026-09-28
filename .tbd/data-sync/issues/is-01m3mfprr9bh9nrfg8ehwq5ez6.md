---
type: is
id: is-01m3mfprr9bh9nrfg8ehwq5ez6
title: Remove the crates.io bootstrap-token path from the publish job
kind: task
status: closed
priority: 3
version: 4
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:06:24.136Z
updated_at: 2026-09-28T23:04:07.582Z
closed_at: 2026-09-28T23:04:07.581Z
close_reason: "Fixed on claude/fdu-alternatives-research-qx0xn0 (PR #155; merges 2d1eea9d, e0923063); CI green at ae650448; release changes independently reviewed (no blocker; low findings in follow-up, signer pinning fdu-8k8s)"
resolution: null
duplicate_of: null
---
release.yml's publish job still prefers a CARGO_REGISTRY_TOKEN secret in the release environment over OIDC ('Choose the crates.io credential', and the two cargo publish steps read secrets.CARGO_REGISTRY_TOKEN || the OIDC token). That path existed only to create fdu-core and fdu for 0.1.0. Both crates now exist, so a secret added later would silently replace trusted publishing with a long-lived token (the job only warns). make release-preflight now fails if such a secret exists, but the workflow itself should not accept one. First confirm both crates list the jlevy/fdu release.yml release trusted publisher on crates.io (not visible without an owner login; the 0.2.0 publish run is the proof). Then drop the secret path and update the test_metadata.py guards that pin exactly three secrets.CARGO_REGISTRY_TOKEN reads. Found in fdu-n2hc.

## Notes

2026-09-28: implemented on claude/fdu-alternatives-research-qx0xn0 (PR #155), merged at 2d1eea9d/e0923063. ca6f4ce1: CARGO_REGISTRY_TOKEN path removed; OIDC only (0.2.0 run 36456542352 proved trusted publishing). Pending: independent review and CI, then close.
