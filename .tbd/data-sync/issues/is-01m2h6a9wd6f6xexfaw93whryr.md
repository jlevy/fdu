---
type: is
id: is-01m2h6a9wd6f6xexfaw93whryr
title: "Release readiness: land the open PR stack and cut the first stable fdu release"
kind: epic
status: in_progress
priority: 0
version: 2
labels:
  - release
dependencies: []
created_at: 2026-09-15T00:08:53.387Z
updated_at: 2026-09-15T00:23:08.855Z
---
User goal (2026-09-14): bring fdu to a stable state that can land on main and cut a release. Keep making progress, track everything as beads, stack PRs as needed, and make sure the final PR stack is complete.

Scope, to be refined by the readiness audit:
1. Land the open PRs: #56 -> #57 -> #60 (GitHub stack), plus #58 and #55. Each must be reviewed through its head, fixes addressed, and CI green.
2. The default-on .gitignore work the user decided (fdu-elnn, fdu-5ryb), with its prerequisites (fdu-1onj, fdu-okne, fdu-szkg), stacked on #60.
3. Release blockers found by the audit: open P0/P1 beads that affect correctness, docs accuracy, CHANGELOG and release notes (fdu-apbl), the version, and the packaging/release workflow rehearsal (docs/project/guides/release-process.md).
4. The final pass: make check and cross-lint on the combined tree, the local release rehearsal, and release notes. The user cuts the release.

Tracking: fdu-6nyd (the earlier merge-readiness bead) covered the first stack, which has merged.

## Notes

2026-09-15 ~00:30 PDT status:
- Open PRs: GitHub stack #59 (#56 -> #57 -> #60), plus #58 and #55, which stand alone on main. Every one is 19/19 green and CLEAN. #58 and #55 are independent of the stack and merge cleanly with it and with each other, so they are deliberately not stacked.
- Review disposition audit:
  - #58: complete, both reviews answered.
  - #60: technical review answered; delta review running.
  - #55: delta review being addressed (fixer running).
  - #56: its technical review's LIFE-1/2/3 and delta review being fixed (fixer running; it must post replies for both).
  - #57: technical review 5200240760 never got a disposition map, and delta review 5204082880 is unanswered. Assigned to the #57 agent with the address-pr-review shortcut.
- Release-readiness audit (Fable) is running; it will create 'release' beads under this epic.
- Next: propagate #56's fixes into #57 and #60, run final delta reviews, then the default-on .gitignore PRs (fdu-elnn, fdu-5ryb, prerequisites fdu-1onj/okne/szkg) stacked on #60, unless the audit recommends deferring them.
