---
type: is
id: is-01m01293x5gaacv3vxjdtrg146
title: Polish fdu 0.1.0 packaging and Python API
kind: epic
status: closed
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01m0129rtmjnmb8y1a4zpwrckp
  - is-01m0129s2s8ctztpscxkjz6vnt
  - is-01m0129sar39fdss9a969ak156
  - is-01m0129sjsp6xht1ae8aebvx0r
  - is-01m0129stsv3wn03hy6m3j4r4t
  - is-01m0129t2wsdsv20mt3bq7s0zh
  - is-01m0142ay3r0xt3823pjarcms6
  - is-01m014dn9rhtwryk6v1e9qnjqs
created_at: 2026-08-14T21:19:05.636Z
updated_at: 2026-08-14T23:53:16.842Z
closed_at: 2026-08-14T23:36:37.139Z
close_reason: "Completed the non-publishing packaging and Python API polish epic: the crate, typed Python package, uvx console, artifact matrix, installed-consumer gates, rehearsal workflow, registry audit, and runbook are implemented and validated. This unblocks fdu-9cf0, which remains open for external account setup, tag, uploads, attestations, GitHub Release, and post-publication verification."
---
Close the release-audit gaps between the working Rust engine and supportable crates.io, PyPI, uvx, and typed Python artifacts. This epic owns the public Python package contract, artifact identity and contents, portable wheel matrix, artifact-first automation, and installed-consumer acceptance gates. The existing fdu-9cf0 bead remains the final publication action and depends on this epic.

## Notes

Implementation completed in PR #26 from branch codex/python-packaging-release-engineering, created directly from origin/main 043e5a7. Scope includes fdu-t5lh, fdu-8d28, fdu-wp21, non-publishing fdu-5eqk, and documentation/rehearsal fdu-lidi. GitHub Actions run 31851444401 passed every executable job, including Rust on Linux/macOS/Windows, MSRV, feature boundaries, Python 3.12/3.14 wheels on all three operating systems, clean source typing/sdist, supply chain, docs, and audits. fdu-9cf0 remains open: no registry mutation, credentials, release tag, upload, attestation, or GitHub Release was performed.
