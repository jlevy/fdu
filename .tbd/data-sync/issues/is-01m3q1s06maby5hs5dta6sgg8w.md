---
type: is
id: is-01m3q1s06maby5hs5dta6sgg8w
title: Document byte-exact .gitignore matching under Unicode normalization on macOS
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T17:00:40.532Z
updated_at: 2026-09-29T17:00:40.532Z
---
fdu matches .gitignore bytes exactly and follows neither git's macOS default core.ignorecase=true nor core.precomposeunicode=true. Say so beside the ignorecase note (control.rs) and pin both off in macOS git differentials.
