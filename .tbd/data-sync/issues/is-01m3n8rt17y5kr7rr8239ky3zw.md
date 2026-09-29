---
type: is
id: is-01m3n8rt17y5kr7rr8239ky3zw
title: Compare fdu's code analysis with tokei, scc, cloc and other SLOC counters
kind: task
status: closed
priority: 2
version: 4
labels:
  - research
  - parity
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-29T00:24:25.383Z
updated_at: 2026-09-29T00:40:00.516Z
closed_at: 2026-09-29T00:40:00.516Z
close_reason: Survey complete; follow-ups fdu-0lo2, fdu-mtdt, fdu-ha14, fdu-bj94.
resolution: null
duplicate_of: null
---
User request 2026-09-29: review the survey for other tools that count source lines and tally them (tokei, scc, cloc, gocloc, loc, polyglot, sloccount, ohcount, linguist, onefetch, pygount), verify their features from source checked out under attic/, and compare fdu's --analyze code: languages, comment/string/nesting rules, generated/vendored detection, .gitignore, per-directory tallies, output formats, cache, APIs, speed (a quiet run on linux-v6.12), accuracy (the 2026-08-13 fixture comparison). Earlier evidence: report-2026-08-13-code-sloc-performance.md (fdu 11.9 ms, scc 9.7 ms, tokei 13.3 ms on a 233-file tree). Deliverable: a brief or a section in the pdu brief, gap beads, and possibly an 'SLOC tools' footnote or row in the README comparison matrix (fdu-dbn9).

## Notes

2026-09-29 maintainer: pick the top 1-2 SLOC tools (speed and power balanced) for the README matrix (fdu-dbn9).

2026-09-29: survey done from attic source (tokei, scc, cloc, gocloc, loc, polyglot, ohcount, linguist, pygount, onefetch). Matrix columns: scc (speed leader, most featureful) and tokei (Rust standard, library, embedded languages); cloc in a footnote. fdu ahead: per-directory tallies (unique), content cache, ignored share per language, coverage reporting, prose metrics, Rust/Python API, agent skill. Behind: 15 languages vs 333-402; no embedded languages, complexity, generated/minified exclusion, duplicates, CSV/HTML/SQL. Follow-ups: fdu-0lo2 (C header probe bug, 168/25,308 kernel headers), fdu-mtdt, fdu-ha14, fdu-bj94 (differential + quiet speed run feeding fdu-dbn9). Full report saved with the matrix draft.
