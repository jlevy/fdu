---
type: is
id: is-01m36t9a6za0cvzfc20wkkhwp3
title: "verify_bead_sync reports four false mismatches (YAML escapes, empty vs null spec_path, notes headed ## Notes)"
kind: bug
status: open
priority: 3
version: 1
labels:
  - tooling
dependencies: []
created_at: 2026-09-23T09:41:55.543Z
updated_at: 2026-09-23T09:41:55.543Z
---
Found following integration runbook section 9 on 2026-09-23: make verify-beads reports 1704/1708. fdu-zrki: the synced title is a YAML double-quoted string with C:\\ (one backslash), which the verifier compares without unescaping. fdu-wfvx and fdu-cggg: an empty local spec_path against null on tbd-sync. fdu-a0cf: a notes body that itself begins with a '## Notes' heading loses that line on the synced side (either the verifier's section split or the tbd format is ambiguous; determine which, since a genuine notes loss would matter). Fix the verifier to parse frontmatter as YAML and normalize empty vs null; investigate the notes case against tbd.
