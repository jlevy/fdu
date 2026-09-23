---
type: is
id: is-01m36bkgrfzj433sbyr81dtbjs
title: "Harness: must_serve passes when oracle and cache read fail identically; stale waiver wording"
kind: task
status: open
priority: 2
version: 1
labels:
  - stack-followup
dependencies: []
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:21.294Z
updated_at: 2026-09-23T05:25:21.294Z
---
Stack review R116-1/2/3 (#116). tests/path_independence/runner.py ~238-245: under must_serve, identical failures of the cold oracle and the only read compare same and pass, so a serving control can pass without serving. Fix: under must_serve require both runs to be non-failures. Reword comments in .github/workflows/path-independence.yml header, ci.yml ~106-109 and the known-violations.toml header that still describe registered violations as acceptable. Reword the overclaiming comment at content_cache.rs 1207-1244.
