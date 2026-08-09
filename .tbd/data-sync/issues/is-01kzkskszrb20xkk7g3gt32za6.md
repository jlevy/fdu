---
type: is
id: is-01kzkskszrb20xkk7g3gt32za6
title: Specify and harden the fdu CLI with golden tests
kind: epic
status: closed
priority: 1
version: 19
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4bey8nn4k8y1daxc9exhd
  - type: blocks
    target: is-01kzg4bf862ajh8g2tmv5bznng
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzksmc3vy1fhq3jea81x1rqm
  - is-01kzksmm19n0zcsefyd4ap44cg
  - is-01kzksmm9990kap5ww6f7gm7ce
  - is-01kzksmmh4vv5hh349wgtdey8w
  - is-01kzksmmryehfvwx8beyyfppg5
  - is-01kzksmvwnqxbgn4x2q9s8avby
  - is-01kzksmw4sw25sx7cnnss31nys
  - is-01kzksn3gepmk01a21gkxxs6bv
  - is-01kzkssg8cxnkr368qyb6dfpjc
  - is-01kzkt8wd2cah0n9kp4h79t8y0
  - is-01kzkt8wd2rwzb1bwyxy7w1weh
  - is-01kzktxd8pfnk7q2e6ndetm3v1
  - is-01kzkvb2y6jhmtzna9e6vh5nxt
created_at: 2026-08-09T17:37:31.127Z
updated_at: 2026-08-09T18:39:20.747Z
closed_at: 2026-08-09T18:39:04.102Z
close_reason: The 25-scenario CLI golden contract is wired into Make and locked audits, its update workflow is proven, and CI run 31329423861 passes it on Linux, macOS, and Windows.
---
Executable CLI contract using four tryscript sessions plus focused platform tests. Covers human and JSON output, errors and exit statuses, cache lifecycle, deterministic cross-platform execution, and every behavior defect recorded in the linked spec.
