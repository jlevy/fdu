---
type: is
id: is-01m0nzxxarrcvcpfpkwz63ca7y
title: "PR #42 review R10: crates/fdu/build.rs says the file-type rules belong to fdu"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:04.376Z
updated_at: 2026-08-23T00:39:56.199Z
closed_at: 2026-08-23T00:39:56.198Z
close_reason: Fixed. The comment now says the rules belong to fdu-core.
---
crates/fdu/build.rs:3-4. This file is the fdu crate; the rules live in fdu-core. Written before the engine was renamed.
