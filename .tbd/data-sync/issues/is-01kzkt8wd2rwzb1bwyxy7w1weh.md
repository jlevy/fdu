---
type: is
id: is-01kzkt8wd2rwzb1bwyxy7w1weh
title: Report tryscript blank-line stderr assertion gap
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies: []
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:49:01.728Z
updated_at: 2026-08-09T17:50:06.021Z
closed_at: 2026-08-09T17:50:06.020Z
close_reason: Filed https://github.com/jlevy/tryscript/issues/45 with a minimal exact-stderr reproduction, parser analysis, workaround tradeoffs, and proposed regression coverage; documented the exact-newline bridge in the fdu spec.
---
File an upstream tryscript issue with a minimal reproduction showing that exact separate-stderr expectations cannot represent blank lines without fragile trailing whitespace. Record the upstream URL and keep the fdu workaround narrowly typed.
