---
type: is
id: is-01m3rhbj53p4j4cghcyd4d3ak0
title: "v0.3.0 release: stabilized, reviewed stack and release PR ready for maintainer approval"
kind: epic
status: open
priority: 0
version: 9
labels: []
dependencies: []
child_order_hints:
  - is-01m3r273jb24qc4hp7ak005jfm
  - is-01m3qgck5yzpd603akhhkpw7t9
  - is-01m3rhbnb9q2my93pfqjz57dxa
  - is-01m3rhbnwhvym3rmjbynhdc4he
  - is-01m3rhbpcafvmgepmaht121ydm
  - is-01m3rhbpw3xpfgrwa8qxs2d9j2
  - is-01m3rhbqca15g81a2qe0j1b65j
  - is-01m3rhbqwjhte75whcntgkr3zc
created_at: 2026-09-30T06:52:11.810Z
updated_at: 2026-09-30T06:52:17.682Z
---
Maintainer request 2026-09-30 night: work autonomously end to end so v0.3.0 is ready to be approved and reviewed in the morning, following up every remaining bug, stability issue and doc issue. Scope (confirmed): #157 -> #158 -> #161 -> #162 -> the stability PR (fdu-l4u1, with atomic writes fdu-3unn) -> #163 (pdu track, fdu-faqa) -> the release PR, one linear stack merged bottom to top with merge commits. Agents may run the release steps that read or can be undone (release-process.md, Who Runs What); tagging, dispatching publication and approving the release environment stay with the maintainer. Deliverables: every PR green, mergeable and senior-reviewed with findings addressed; the release PR (version 0.3.0, dated CHANGELOG, release notes 0.3.0.md); the stability pass on the candidate (make check, cross-lint, semver-check, release-rehearse, installed-CLI QA with peer agreement, correctness runbook) recorded; a morning readiness summary.
