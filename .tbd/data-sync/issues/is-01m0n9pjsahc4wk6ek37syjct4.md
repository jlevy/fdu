---
type: is
id: is-01m0n9pjsahc4wk6ek37syjct4
title: Close the Python/CLI parity gaps the harness found
kind: epic
status: open
priority: 1
version: 9
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
created_at: 2026-08-22T17:53:35.529Z
updated_at: 2026-08-22T18:31:00.447Z
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
