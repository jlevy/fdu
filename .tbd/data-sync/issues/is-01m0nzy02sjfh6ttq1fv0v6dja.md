---
type: is
id: is-01m0nzy02sjfh6ttq1fv0v6dja
title: "PR #42 review R17: run-parity.mjs tests absoluteness with startsWith('/')"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:07.193Z
updated_at: 2026-08-23T00:39:58.431Z
closed_at: 2026-08-23T00:39:58.431Z
close_reason: Fixed. run-parity.mjs uses path.isAbsolute.
---
scripts/run-parity.mjs:61. A Windows absolute path would be joined onto the repo root. Use path.isAbsolute.
