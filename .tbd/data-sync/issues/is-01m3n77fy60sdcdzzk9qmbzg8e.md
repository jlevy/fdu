---
type: is
id: is-01m3n77fy60sdcdzzk9qmbzg8e
title: "README: add a Comparison to Alternatives matrix (fdu vs du, dust, pdu, dumac, and peers)"
kind: task
status: open
priority: 1
version: 7
labels:
  - docs
  - parity
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T23:57:29.414Z
updated_at: 2026-09-29T00:24:50.087Z
---
Add a "Comparison to Alternatives" matrix to the main README, modeled on https://github.com/jlevy/repren#comparison-to-alternatives: tools as columns, features as rows, each cell a ✅ / ❌ or a few words ("partial: total only", "optional flag"), one intro sentence above, and a short "when to use each" note below that says plainly where a peer is the better choice.

Tools (columns; confirm the final set): fdu, GNU du, ncdu, dust, dua, gdu, pdu, diskus, dumac; consider scc or tokei as the reference for the code-analysis row only, or name them in a footnote, since they are not disk-usage tools.

Feature rows (from the maintainer, 2026-09-28), each orthogonal:
1. Plain usage: one total for a tree.
2. Speed: plain total and the basic tree, with the platform and figure from the published comparisons (macOS and Linux reports) rather than an adjective.
3. Tree breakdowns with flexible pruning: depth, breadth, share floor, row limits.
4. .gitignore support: classify, include, exclude, or show only ignored entries.
5. Source code analysis: languages, SLOC, comments, blanks.
6. Text file analysis: words, pages, paragraphs for Markdown and text.
7. APIs beyond the command line: list them (fdu: Rust fdu-core and the Python package; pdu: Rust library crate; dua: crates; diskus: small Rust library).
8. Watch and streaming support.
9. Cache optimizations for computed numbers (content sidecar, snapshot).
10. Agent-friendly skill.

Requirements:
- Every cell is checked against the tool's source or its current docs, not recalled. The pdu brief (docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md, section E) already has the pdu parity; the 2026-08-06 survey matrix and the 2026-09-25 peer-agreement report cover the others. Cite versions checked in a footnote.
- The speed row states measured results with their conditions and links the reports. Where fdu is not first (on Linux today: the indexed tree against pdu and diskus, and the default command on source trees with many .gitignore files, about 3x pdu after H162/H163), the cell says so, or the work that closes it (H164, H166, H167) lands first and a fresh quiet matrix supports the claim. The README must not claim fdu is fastest where the evidence does not show it.
- Keep it scannable: at most about 10 rows and 10 columns; details in footnotes.
- Run flowmark on README.md; keep the Why section consistent with the matrix (it lists fifteen surveyed tools).

## Notes

2026-09-28 maintainer: cells are a checkmark, an x, or concise text, whichever is clearer for that cell (e.g. '✅', '❌', 'total only', 'optional flag', 'Rust, Python').

2026-09-28 maintainer: add an eleventh row, Platform support, listing platforms per tool (fdu: macOS, Linux, Windows; dumac: macOS only; others to be verified from each project's releases and docs, e.g. prebuilt binaries vs builds from source).

2026-09-29 maintainer: add a twelfth row, Installation: the install mechanisms per tool (cargo crate, Homebrew, uv/pip, apt and other distro packages, go install, GitHub release binaries, ...), whether each is prebuilt or builds from source, and for fdu that it runs with zero install via uvx (uvx --no-build fdu@latest). Verify each tool's mechanisms from its README and package listings.

2026-09-29 maintainer: add a thirteenth row, Output formats: list each tool's formats (fdu: text, colored terminal, tree, paths, long, JSON, JSONL, YAML — verify the exact list from docs/usage.md and the CLI), and say whether terminal output is colored where supported (e.g. dust and gdu color; confirm per tool).

2026-09-29 maintainer: every cell is verified from source checked out under attic/ (gitignored), not from web search alone; web pages only for packaging and distribution facts not in the repos. Research running in three parallel agents (du/ncdu/gdu; dust/dua/pdu; diskus/dumac/fdu).

2026-09-29 maintainer: add the top 1 or 2 source-line-counting tools as extra matrix columns (covering the source-code rows; other rows n/a or as applicable), chosen by balancing speed and power; selection from the fdu-61ez survey plus a quiet speed run on linux-v6.12.
