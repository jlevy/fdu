---
type: is
id: is-01kzz31zpz1y045pvjf6k77ccw
title: Decide how the supply-chain audit should honour first-party PyPI exemptions
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:54:11.679Z
updated_at: 2026-08-14T02:54:11.679Z
---
Extending scripts/check-supply-chain.mjs to benchmarks/uv.lock immediately surfaced softschema 0.6.0 and its dependency frontmatter-format 0.4.0 inside the 14-day cool-off. Both are first-party, and AGENTS.md plus SUPPLY-CHAIN-SECURITY.md already say first-party packages are exempt because the cool-off exists to let somebody else notice a compromised upstream release, which does not apply to a package this project's own authors publish.

The audit does not know that. It re-implements the cool-off independently and reads none of the exclude-newer-package tables that record the exemption, so every first-party PyPI release now needs a hand-written expiring record in supply-chain-policy.json. Two such records are in place and expire on 2026-08-16 and 2026-08-19.

There is also a mismatch worth a maintainer's eye: benchmarks/uv.toml declares only softschema, while the [options.exclude-newer-package] block uv wrote into both lockfiles names nineteen first-party packages. The committed policy and the config that actually resolved the locks are not the same.

Options, for the maintainer rather than for an agent to pick:
1. Teach the audit to read [exclude-newer-package] from the committed uv.toml beside each lock, and declare first-party transitive dependencies there. Keeps the audit and the resolver reading one policy; widens what a uv.toml edit can waive.
2. Keep hand-written exceptions and accept the recurring toil on each first-party release.
3. Reconcile benchmarks/uv.toml with the nineteen names in the lock options block, so the committed policy matches what resolved the lock.

Read tbd guidelines supply-chain-hardening before changing any of this.
