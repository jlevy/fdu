---
type: is
id: is-01m0142ay3r0xt3823pjarcms6
title: Incorporate Flowmark release practices into the fdu packaging plan
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels: []
dependencies: []
parent_id: is-01m01293x5gaacv3vxjdtrg146
created_at: 2026-08-14T21:50:20.610Z
updated_at: 2026-08-14T22:02:26.965Z
closed_at: 2026-08-14T22:02:26.964Z
close_reason: Fetched and reviewed immutable Flowmark v0.3.2 and current Rust porting playbook references; verified the production release channels; incorporated adopted practices, deliberate mixed-PyO3 and security divergences, same-owner project-specific credential setup, release workflow details, gates, risks, and upstream candidates into the packaging/API plan. Follow-up upstream work is tracked by fdu-8bn9. Full make check passes.
---
Review fetched flowmark-rs origin/main and the current Rust porting playbook, compare their proven packaging, release, credential, CI, and versioning patterns with fdu, record direct adoptions and justified divergences, and add explicit upstream improvement candidates without changing either reference repository.

## Notes

Reviewed fetched Flowmark origin/main at 015f23989af3e5cfb3f8b58dfc72822c534df25a (v0.3.2) and Rust porting playbook main at d24760a3fbd2951c730a199269aeb082abb46a42. Verified Flowmark's GitHub release, crates.io crate, PyPI matrix, and successful release run. Plan now records adopted patterns, mixed-PyO3 and security divergences, same-owner/project-specific OIDC setup, and tracked upstream fixes in fdu-8bn9.
