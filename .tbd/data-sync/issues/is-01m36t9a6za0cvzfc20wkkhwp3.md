---
type: is
id: is-01m36t9a6za0cvzfc20wkkhwp3
title: "verify_bead_sync reports four false mismatches (YAML escapes, empty vs null spec_path, notes headed ## Notes)"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - tooling
dependencies: []
created_at: 2026-09-23T09:41:55.543Z
updated_at: 2026-09-30T10:08:53.763Z
closed_at: 2026-09-30T10:08:53.763Z
close_reason: |
  Fixed on claude/stability-tooling in 6ff2d880. All four were verifier misreadings, none a sync loss: fdu-zrki (double-quoted `C:\\` compared without YAML unescaping), fdu-wfvx and fdu-cggg (empty local spec_path against the `null` tbd writes), fdu-a0cf (notes opening with their own `## Notes` heading: tbd splits at the first such heading and keeps the rest, so the synced notes are intact; the verifier split at every heading). The verifier now parses frontmatter as YAML in the subset tbd 0.9.0 writes (stdlib only, raising on anything outside it) and splits the body as tbd's parseMarkdownWithFrontmatter does. Evidence: `python3 scripts/verify_bead_sync.py fdu-zrki fdu-wfvx fdu-cggg fdu-a0cf` gives 4/4 ok (previous verifier: 1/4); a full run over 2067 beads reports no false mismatch, every remaining difference being a bead created or edited after the last sync (local updated_at newer than synced). tests/release/test_verify_bead_sync.py (8 tests) covers the four cases; make release-test (223), make atomic-writes, make docs-format-check pass.
resolution: null
duplicate_of: null
---
Found following integration runbook section 9 on 2026-09-23: make verify-beads reports 1704/1708. fdu-zrki: the synced title is a YAML double-quoted string with C:\\ (one backslash), which the verifier compares without unescaping. fdu-wfvx and fdu-cggg: an empty local spec_path against null on tbd-sync. fdu-a0cf: a notes body that itself begins with a '## Notes' heading loses that line on the synced side (either the verifier's section split or the tbd format is ambiguous; determine which, since a genuine notes loss would matter). Fix the verifier to parse frontmatter as YAML and normalize empty vs null; investigate the notes case against tbd.
