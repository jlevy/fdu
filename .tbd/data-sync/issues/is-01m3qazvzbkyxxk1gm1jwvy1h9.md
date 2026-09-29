---
type: is
id: is-01m3qazvzbkyxxk1gm1jwvy1h9
title: "Modeline probe matches a language alias as a prefix: 'mode: conf-colon' classifies a file as C"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T19:41:42.762Z
updated_at: 2026-09-29T19:41:42.762Z
---
`modeline_rule` in crates/fdu-core/src/classify/file_type_detection.rs tests `lower.contains(&format!("mode: {alias}"))`, `ft={alias}`, and `filetype={alias}` as plain substrings, so an alias matches as a prefix of a longer mode name: `# -*- mode: conf-colon -*-` and `# -*- mode: conf -*-` classify a file as C (alias `c`), and `ft=c` would match `ft=cs`, `ft=css`, or `ft=cmake`; `mode: go` matches `mode: gomod`.

Found by the SLOC differential (fdu-bj94): Linux v6.12's `Documentation/docutils.conf` (`# -*- coding: utf-8 mode: conf-colon -*-`) is counted as C with 6 code lines. Reproduction: a file `a.conf` containing `# -*- mode: conf-colon -*-` then `[general]`; `fdu . --analyze=code --view=files --format=json` reports file_type c, source modeline.

Fix: match the alias as a whole token (end at `;`, whitespace, `-*-`, `:`, or end of line), and add fixtures for `conf`, `conf-colon`, `cs`, `css`, `cmake`, `gomod`, plus the existing positive cases. Same class of defect as fdu-0lo2 (substring probe without a token boundary). Changes classification only; line counts of genuinely detected files are unaffected.
