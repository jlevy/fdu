---
type: is
id: is-01m36bkdy0gnqnn7z6jpd41ggv
title: YAML plain policy leaks YAML 1.1 exponent scalars (e3, E10, e+5) as numbers
kind: bug
status: open
priority: 1
version: 2
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:18.399Z
updated_at: 2026-09-23T05:25:51.316Z
---
Found in the 2026-09-22 stack review (R113-2), introduced in #113, still present at #117 adc39d24. crates/fdu-core/src/emit/emit_scalar.rs is_plain_safe keeps letter-initial alphanumerics plain. The gate parser (node_modules/yaml, version 1.1) resolves e3/E10/e+5/e-1 as numbers (NaN) while 1.2 resolves strings, verified by running it. A file or directory named e3 breaks YAML 1.1 = 1.2 = JSON. Fix: quote strings matching ^[-+]?([0-9][0-9_]*)?(\.[0-9_]*)?[eE][-+]?[0-9]+$; add e3, E+3, e-1 to CORPUS and both corpus fixtures (Rust test and scripts/check-yaml.mjs). Fix on #113 and merge up.
