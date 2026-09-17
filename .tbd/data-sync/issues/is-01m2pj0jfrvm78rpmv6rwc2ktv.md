---
type: is
id: is-01m2pj0jfrvm78rpmv6rwc2ktv
title: Release automation after 0.1.0
kind: epic
status: open
priority: 2
version: 12
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
  - packaging
dependencies: []
child_order_hints:
  - is-01m2pj0k3cbceh1mzza485t6pr
  - is-01m2pj0kq0s5533yhbzfetepbv
  - is-01m2pj0md1j5nsjfay2prdr5hc
  - is-01m2pj0mxyy7bem62kw1vdzpaw
  - is-01m2pj0ngk9sgeppchy5zd6z54
  - is-01m2pj0p31w8pg5wahbayvvy8d
  - is-01m2pj0pn53jgvgbty93drg2zh
  - is-01m2pj0q04xsqtjqj2v62vhhq5
  - is-01m2pj0qb7ahjpnbfth4kfdpp7
  - is-01m2pj0qp3y41g5rcs99bm5fc7
  - is-01m2pj0rjcedzrm53gsh4jawny
created_at: 2026-09-17T02:09:29.592Z
updated_at: 2026-09-17T02:09:35.820Z
---
Workflow publication for later releases, in the order the packaging audit proposed: protected release
environment, trusted publishers (PyPI; crates.io once the crates exist), release mode in release.yml with
signed-tag verification, crates.io and PyPI OIDC publish jobs over the validated artifacts, provenance
attestations, a partial registry state, a GitHub Release job, semver checks, 0.1.1 packaging hygiene, and
a free-threaded CPython decision.
