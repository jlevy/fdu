---
type: is
id: is-01m0nzxzan58earxsdzzw21hpv
title: "PR #42 review R15: fdu-core keeps CLI keywords and category after the split"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:06.421Z
updated_at: 2026-08-23T00:39:57.796Z
closed_at: 2026-08-23T00:39:57.796Z
close_reason: Fixed. fdu-core keywords and categories describe an engine; cli and command-line-utilities stay on fdu.
---
crates/fdu-core/Cargo.toml:7-8 still list keyword 'cli' and category 'command-line-utilities'. Those describe fdu now.
