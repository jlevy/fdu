---
type: is
id: is-01kzksmc3vy1fhq3jea81x1rqm
title: Add locked tryscript harness and deterministic CLI fixture
kind: task
status: closed
priority: 1
version: 9
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksmm19n0zcsefyd4ap44cg
  - type: blocks
    target: is-01kzksmm9990kap5ww6f7gm7ce
  - type: blocks
    target: is-01kzksmmh4vv5hh349wgtdey8w
  - type: blocks
    target: is-01kzksmmryehfvwx8beyyfppg5
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:37:49.690Z
updated_at: 2026-08-09T18:39:21.121Z
closed_at: 2026-08-09T17:43:31.577Z
close_reason: Pinned tryscript 0.1.7 with audited lockfile and lifecycle scripts disabled; added an isolated exact-output smoke transcript, portable fixture tree, and local run/update scripts. npm ci, npm audit, golden smoke, and make check all pass.
---
Pin tryscript 0.1.7 with a committed npm lockfile and disabled lifecycle scripts; add shared deterministic config, the one portable fixture tree, and local run/update commands. Acceptance: the built target/debug/fdu is discoverable on every supported OS and an exact smoke transcript runs in an isolated sandbox.
