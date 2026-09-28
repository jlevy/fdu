---
type: is
id: is-01m3mfprr9bh9nrfg8ehwq5ez6
title: Remove the crates.io bootstrap-token path from the publish job
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m3mcwy5q1j1nab2srxz38ebb
created_at: 2026-09-28T17:06:24.136Z
updated_at: 2026-09-28T17:06:24.136Z
---
release.yml's publish job still prefers a CARGO_REGISTRY_TOKEN secret in the release environment over OIDC ('Choose the crates.io credential', and the two cargo publish steps read secrets.CARGO_REGISTRY_TOKEN || the OIDC token). That path existed only to create fdu-core and fdu for 0.1.0. Both crates now exist, so a secret added later would silently replace trusted publishing with a long-lived token (the job only warns). make release-preflight now fails if such a secret exists, but the workflow itself should not accept one. First confirm both crates list the jlevy/fdu release.yml release trusted publisher on crates.io (not visible without an owner login; the 0.2.0 publish run is the proof). Then drop the secret path and update the test_metadata.py guards that pin exactly three secrets.CARGO_REGISTRY_TOKEN reads. Found in fdu-n2hc.
