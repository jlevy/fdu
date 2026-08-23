---
type: is
id: is-01m0nzxthn96qvqjy6wevaf0my
title: "PR #42 review R3: fdu-core README documents the wrong crate and the deleted cli feature"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:01.525Z
updated_at: 2026-08-23T00:39:53.924Z
closed_at: 2026-08-23T00:39:53.923Z
close_reason: "Fixed. crates/fdu-core/README.md rewritten for the engine: title, fdu-core dependency form, fdu_core:: in the example, the removed cli feature dropped, and a paragraph saying fdu is what you install."
---
crates/fdu-core/README.md moved with a zero-line diff. It is fdu-core's published readme but is titled '# fdu', shows fdu = { path = "crates/fdu" }, uses fdu:: in the example, and states 'Features: cli and watch are enabled by default' -- naming the feature this PR removes.
