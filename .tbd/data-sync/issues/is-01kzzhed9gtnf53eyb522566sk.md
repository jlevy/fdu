---
type: is
id: is-01kzzhed9gtnf53eyb522566sk
title: "Adopt tbd 0.5.0: update the pinned bootstrap provenance"
kind: chore
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-14T07:05:38.864Z
updated_at: 2026-08-14T07:09:50.281Z
closed_at: 2026-08-14T07:09:50.281Z
close_reason: "Done rather than deferred: tbd is first-party and exempt from the cool-off per AGENTS.md, so the bootstrap pin moves to get-tbd@0.6.0 with the integrity and tarball read from the registry, and the first-party exception record moves with it. The 0.4.2 exception was expiring 2026-08-15 anyway."
---
The session-start hook installs the current tbd, so a repo pinned to an older bootstrap drifts from the tool actually running. Running 'tbd setup --auto' regenerates .claude/ and .codex/ hooks and skills to reference get-tbd@0.5.0, which the supply-chain check then rejects: the bootstrap pin in the policy carries a recorded npm integrity hash and tarball URL for 0.4.2, and those must be re-recorded and verified against the registry for 0.5.0. That is the documented process working as intended rather than an obstacle, so the bump wants its own change: follow SUPPLY-CHAIN-SECURITY.md and 'tbd guidelines supply-chain-hardening', record the new integrity and tarball provenance, then commit the regenerated scaffolding alongside it. Attempted accidentally on the Linux performance branch (commit e650518) while looking for the gh install script, and reverted in a5be1fe because it is unrelated to that work and needs the review. 0.5.0 is worth taking: it adds 'tbd web' and, usefully for these remote sessions, skill guidance that gh works through a scoped NO_PROXY bypass when the environment has egress.
