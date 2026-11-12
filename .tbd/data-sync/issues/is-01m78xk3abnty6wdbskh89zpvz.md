---
type: is
id: is-01m78xk3abnty6wdbskh89zpvz
title: "release-demo: retry the draft lookup after creating it"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-11-12T22:20:58.059Z
updated_at: 2026-11-12T22:20:58.059Z
---
make release-demo (scripts/release/maintainer.py demo) creates the draft with gh release create and immediately lists releases to read it back. On 0.6.0 (2026-11-12) the listing did not yet include the new draft, so the step failed with 'GitHub did not return the draft release v0.6.0 just created'; a rerun a minute later found the draft and attached the video. The releases listing is eventually consistent after a create. Retry release_record() a few times with a short backoff before raising, and cover it with a test that returns None on the first read.
