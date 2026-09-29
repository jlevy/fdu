---
type: is
id: is-01m3nedew24rmcnjfc97jtf0z4
title: release preflight crashes when gh is not installed instead of failing its gh lines
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T02:03:04.962Z
updated_at: 2026-09-29T02:03:04.962Z
---
scripts/release/maintainer.py preflight promises 'One unreadable source fails its own line, not the whole checklist', but Host.run raises FileNotFoundError (not CommandError) when the gh binary is absent, so the whole step dies with a traceback at reporting_check. Catch FileNotFoundError in Host.run (convert to CommandError with 'gh not installed'), add a test with a Host whose PATH lacks gh. Found running the 0.2.1 preflight from a session without gh (2026-09-29).
