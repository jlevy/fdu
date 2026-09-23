---
type: is
id: is-01m2pmrn6ka9kt7f4kcjcmvn8c
title: "Close conformance: empty the violation registry and remove known-gaps sections"
kind: task
status: in_progress
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:35.954Z
updated_at: 2026-09-23T04:50:53.217Z
---
Acceptance for the plan: path-independence, metric-independence, and writer-equality tests pass on Linux, macOS,
and Windows with an empty registry; no request field parsed, defaulted, or validated outside the request model; no
route decides reads or writes outside the plan model; every tier has identity, fingerprint, serves/project;
sessions and Python Index compute provenance; design principles lose the conformance pointer and architecture
documents lose their known-gaps sections.

## Notes

Local macOS gate evidence (2026-09-22): runtime at 4464308e. Initial `make check` passed the pre-lib-only targets (including Rust tests, CLI goldens, docs, and performance evidence), then stopped at the opened-root session golden because fixture 74 was integrated during that run; the fixture was subsequently added and the opened-root golden test passed. A scoped rerun covered the remaining targets on ff2b07da's tracked tree (6cf69217a69ba9198196cfff6687cf587e98017c): `make lib-only msrv audit npm-audit python-check python-concurrency python-smoke python-sdist-smoke parity-check test-path-independence path-independence release-test` exited 0. This included 750 featureless and 829 watch core tests, 67 Python tests, clean wheel and sdist smoke, 24 classified parity deviations, 2,030 path-independence subset cases, and 44 release tests. `make cross-lint` exited 0 for both x86_64-apple-darwin and x86_64-pc-windows-msvc. Latest commit 8edd9b21 differs from ff2b07da only by numeric-literal formatting in one golden-support test; that exact watch-gated normalization test passed at 8edd9b21. All local `make check` targets passed across the scoped runs, rather than in one uninterrupted command. Publication, remote CI, and merge acceptance remain pending.

2026-09-22 final exact-head evidence: PR117 adc39d24 full path-independence run 35819047425 passed 46,926 cases (Linux 16,272, macOS 16,272, Windows 14,382), with zero false verdicts, zero exception lists, and an empty registry on each platform. Current-head CI is green on PR113 876bd39c, PR114 0266b95c, PR115 eb862ec6, PR116 57f01a48, and PR117 adc39d24 (19/19 each). PR115 prior Windows stall cause remains unproven; test-only unwind cleanup guard fdu-ex5k passed fresh Windows CI. Acceptance remains in progress until merge and final spec/document reconciliation.
