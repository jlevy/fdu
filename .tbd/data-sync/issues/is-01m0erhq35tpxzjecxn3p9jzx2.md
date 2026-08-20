---
type: is
id: is-01m0erhq35tpxzjecxn3p9jzx2
title: "Epic: end-to-end verification and text view headers"
kind: epic
status: closed
priority: 2
version: 7
labels: []
dependencies: []
child_order_hints:
  - is-01m0erjy8s1zecx68ymry48fak
  - is-01m0erjyqcz6t04k8k9k4btc9p
  - is-01m0ethvrzj0kgm3wzyvevnq4p
  - is-01m0etj474fxevzjn53r81ct15
  - is-01m0etp7wf33svtvj1y4y8mxte
created_at: 2026-08-20T04:58:23.706Z
updated_at: 2026-08-20T06:32:46.565Z
closed_at: 2026-08-20T06:32:46.564Z
close_reason: "All child work complete and shipped in PR #34: view headers (fdu-dzhm), the end-to-end round (fdu-ko2g), and the three bugs it found (fdu-muzk, fdu-2toe, fdu-hs7q). All 17 CI checks green on macOS, Linux, and Windows."
---
Full round of end-to-end testing of fdu on macOS to confirm consistent behavior, plus a text-format change: label each view with an all-caps header (colored when color is on) separated by a blank line, so multiple views no longer concatenate ambiguously. Structured formats (JSON/JSONL/YAML) are unchanged.
