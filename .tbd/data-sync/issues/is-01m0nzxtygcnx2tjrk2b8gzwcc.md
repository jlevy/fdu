---
type: is
id: is-01m0nzxtygcnx2tjrk2b8gzwcc
title: "PR #42 review R4: benchmarks/README.md documents a perf_probe build command that now fails"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:01.936Z
updated_at: 2026-08-23T00:39:54.235Z
closed_at: 2026-08-23T00:39:54.235Z
close_reason: Fixed. benchmarks/README.md now builds the probe with -p fdu-core.
---
benchmarks/README.md:79 still says -p fdu --example perf_probe. The example moved to fdu-core; Makefile and ci.yml were updated, this file was not.
