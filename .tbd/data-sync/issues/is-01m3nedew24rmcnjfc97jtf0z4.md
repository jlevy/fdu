---
type: is
id: is-01m3nedew24rmcnjfc97jtf0z4
title: release preflight crashes when gh is not installed instead of failing its gh lines
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T02:03:04.962Z
updated_at: 2026-09-30T02:46:07.958Z
closed_at: 2026-09-30T02:46:07.957Z
close_reason: "Fixed in 647afcbc: Host.run converts FileNotFoundError into CommandError (status 127, '<prog>: not installed (not on PATH)'); Host.attach prints it and returns 127. New tests: preflight with gh run on a PATH lacking gh fails exactly its four gh-backed lines (reproduced the traceback before the fix), and HostTests covers run/attach. make release-test: 188 tests OK."
resolution: null
duplicate_of: null
---
scripts/release/maintainer.py preflight promises 'One unreadable source fails its own line, not the whole checklist', but Host.run raises FileNotFoundError (not CommandError) when the gh binary is absent, so the whole step dies with a traceback at reporting_check. Catch FileNotFoundError in Host.run (convert to CommandError with 'gh not installed'), add a test with a Host whose PATH lacks gh. Found running the 0.2.1 preflight from a session without gh (2026-09-29).
