---
type: is
id: is-01m4vh1nmbs96hmr511stktf8r
title: Audit the release pipeline's SHA-256 checks against the hashing-ceremony rule
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-13T21:00:12.811Z
updated_at: 2026-10-13T21:00:12.811Z
---
general-coding-rules (Cryptographic Hash Checks) says a hash comparison adds assurance only when the two sides are computed independently with data outside our control in between. The 0.5.0 release ran several that fail that test. make release-candidate downloads the rehearsal's artifacts and re-verifies them against SHA256SUMS from the same run, over the same channel, moments later; one process computes both sides, so the check can only catch bugs in itself. Probably the same for verifications inside release.yml where one run computes and then checks its own manifest. Keep the checks that cross a real boundary: the demo video against its committed declaration, and registries against the publish run (make release-published). For each remaining SHA-256 step in scripts/release/ and .github/workflows/release.yml, name the failure or tampering it catches, or remove it, and simplify the manifest and schema slots that exist only to carry those digests. Raised by the maintainer during the 0.5.0 release.
