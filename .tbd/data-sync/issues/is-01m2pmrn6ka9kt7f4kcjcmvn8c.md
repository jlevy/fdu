---
type: is
id: is-01m2pmrn6ka9kt7f4kcjcmvn8c
title: "Close conformance: empty the violation registry and remove known-gaps sections"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:35.954Z
updated_at: 2026-09-23T04:02:52.713Z
---
Acceptance for the plan: path-independence, metric-independence, and writer-equality tests pass on Linux, macOS,
and Windows with an empty registry; no request field parsed, defaulted, or validated outside the request model; no
route decides reads or writes outside the plan model; every tier has identity, fingerprint, serves/project;
sessions and Python Index compute provenance; design principles lose the conformance pointer and architecture
documents lose their known-gaps sections.

## Notes

Local macOS gate evidence (2026-09-22): runtime at 4464308e. Initial `make check` passed the pre-lib-only targets (including Rust tests, CLI goldens, docs, and performance evidence), then stopped at the opened-root session golden because fixture 74 was integrated during that run; the fixture was subsequently added and the opened-root golden test passed. A scoped rerun covered the remaining targets on ff2b07da's tracked tree (6cf69217a69ba9198196cfff6687cf587e98017c): `make lib-only msrv audit npm-audit python-check python-concurrency python-smoke python-sdist-smoke parity-check test-path-independence path-independence release-test` exited 0. This included 750 featureless and 829 watch core tests, 67 Python tests, clean wheel and sdist smoke, 24 classified parity deviations, 2,030 path-independence subset cases, and 44 release tests. `make cross-lint` exited 0 for both x86_64-apple-darwin and x86_64-pc-windows-msvc. Latest commit 8edd9b21 differs from ff2b07da only by numeric-literal formatting in one golden-support test; that exact watch-gated normalization test passed at 8edd9b21. All local `make check` targets passed across the scoped runs, rather than in one uninterrupted command. Publication, remote CI, and merge acceptance remain pending.
