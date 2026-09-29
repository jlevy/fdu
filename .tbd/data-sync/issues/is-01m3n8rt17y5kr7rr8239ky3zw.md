---
type: is
id: is-01m3n8rt17y5kr7rr8239ky3zw
title: Compare fdu's code analysis with tokei, scc, cloc and other SLOC counters
kind: task
status: open
priority: 2
version: 2
labels:
  - research
  - parity
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-29T00:24:25.383Z
updated_at: 2026-09-29T00:24:51.217Z
---
User request 2026-09-29: review the survey for other tools that count source lines and tally them (tokei, scc, cloc, gocloc, loc, polyglot, sloccount, ohcount, linguist, onefetch, pygount), verify their features from source checked out under attic/, and compare fdu's --analyze code: languages, comment/string/nesting rules, generated/vendored detection, .gitignore, per-directory tallies, output formats, cache, APIs, speed (a quiet run on linux-v6.12), accuracy (the 2026-08-13 fixture comparison). Earlier evidence: report-2026-08-13-code-sloc-performance.md (fdu 11.9 ms, scc 9.7 ms, tokei 13.3 ms on a 233-file tree). Deliverable: a brief or a section in the pdu brief, gap beads, and possibly an 'SLOC tools' footnote or row in the README comparison matrix (fdu-dbn9).

## Notes

2026-09-29 maintainer: pick the top 1-2 SLOC tools (speed and power balanced) for the README matrix (fdu-dbn9).
