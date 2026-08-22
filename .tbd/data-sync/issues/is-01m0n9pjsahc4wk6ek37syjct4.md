---
type: is
id: is-01m0n9pjsahc4wk6ek37syjct4
title: Close the Python/CLI parity gaps the harness found
kind: epic
status: closed
priority: 1
version: 11
labels: []
dependencies: []
child_order_hints:
  - is-01m0n9q2p70kjhb4dgaek1v0mj
  - is-01m0n9q31gpp0p4sqge8t64hqb
  - is-01m0n9q3by1dk5kcqdzyb7n4cw
  - is-01m0n9sbn9ek4bgz8m8fcw767t
  - is-01m0n9sc2syypwvd87kjfztj9g
  - is-01m0n9sccqby31wvdb6d9559qr
  - is-01m0nbkjftj3mzm51f91t2nwdb
  - is-01m0nbv330xhagsyc2ehtm7hha
  - is-01m0ne12cwet7nwvwhrfgt31nr
created_at: 2026-08-22T17:53:35.529Z
updated_at: 2026-08-22T21:48:01.188Z
closed_at: 2026-08-22T21:48:01.187Z
close_reason: "Every gap the harness found is closed. 108 of 126 sessions reach parity, and the 18 that differ are four named classes, none of them defects: each surface naming its own parameter (11), notes carrying walk telemetry the schema excludes (3), one rule in each surface's own knob names (2), and two discovery surfaces (2). An unexplained difference fails the run, so that list is what is known rather than what was tolerated."
---
The parity harness records 39 deviations across 126 sessions. Triage sorts them into seven causes, all but two fixable. Package is unreleased, so no backward compatibility constrains the fixes.

Sessions by cause:
  9  cache lifecycle has no renderer (fdu-1kw3)
  8  diagnostics: wording and flag prefixes differ
  6  report envelope differs (schema version, root)
  3  display notes missing from Report.render
  2  watch records have no renderer (fdu-1kw3)
  2  filesystem error format differs
  2  list grammar not exposed (fdu-jozr)
  1  depth default lives in the CLI, not the library
  2  deliberate (--version names the surface; --docs is declined)

The recurring root cause is the one this epic keeps hitting: things the library should own live in cli.rs or are hand-copied into the binding, so the two surfaces drift and no test notices.
