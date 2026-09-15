---
type: is
id: is-01m2h6a9wd6f6xexfaw93whryr
title: "Release readiness: land the open PR stack and cut the first stable fdu release"
kind: epic
status: open
priority: 0
version: 1
labels:
  - release
dependencies: []
created_at: 2026-09-15T00:08:53.387Z
updated_at: 2026-09-15T00:08:53.387Z
---
User goal (2026-09-14): bring fdu to a stable state that can land on main and cut a release. Keep making progress, track everything as beads, stack PRs as needed, and make sure the final PR stack is complete.

Scope, to be refined by the readiness audit:
1. Land the open PRs: #56 -> #57 -> #60 (GitHub stack), plus #58 and #55. Each must be reviewed through its head, fixes addressed, and CI green.
2. The default-on .gitignore work the user decided (fdu-elnn, fdu-5ryb), with its prerequisites (fdu-1onj, fdu-okne, fdu-szkg), stacked on #60.
3. Release blockers found by the audit: open P0/P1 beads that affect correctness, docs accuracy, CHANGELOG and release notes (fdu-apbl), the version, and the packaging/release workflow rehearsal (docs/project/guides/release-process.md).
4. The final pass: make check and cross-lint on the combined tree, the local release rehearsal, and release notes. The user cuts the release.

Tracking: fdu-6nyd (the earlier merge-readiness bead) covered the first stack, which has merged.
