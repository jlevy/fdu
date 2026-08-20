---
type: is
id: is-01m0erjyqcz6t04k8k9k4btc9p
title: "Text format: all-caps view headers"
kind: feature
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0erhq35tpxzjecxn3p9jzx2
created_at: 2026-08-20T04:59:04.299Z
updated_at: 2026-08-20T05:51:41.199Z
closed_at: 2026-08-20T05:51:41.188Z
close_reason: "Shipped in PR #34 (https://github.com/jlevy/fdu/pull/34). Multi-view text reports label each block with an all-caps header directly above its rows, blank line between blocks, bold cyan when color is on. Conditional on more than one view so single-view reports stay byte-identical and --view files remains pipeable. Machine formats untouched. Covered by 4 unit tests, 1 process-boundary test, and 4 golden sessions; all 17 CI checks green on macOS, Linux, and Windows."
---
In Format::Text only, prefix each rendered section with an all-caps header naming the view (TREE, TYPES, LANGUAGES, ...), colored when color is enabled, always shown, with a blank line between views. JSON/JSONL/YAML output is untouched. Update goldens and docs.
