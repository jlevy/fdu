---
type: is
id: is-01m2ebc7j9b7w94d6y9jp0wwgp
title: "PR #49 review FLOOR-9: malformed instrument output raises a traceback, not a refusal"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:36.136Z
updated_at: 2026-09-13T22:02:35.400Z
closed_at: 2026-09-13T22:02:35.399Z
close_reason: "Fixed: Instrument.read wraps IndexError/KeyError/JSONDecodeError/TypeError/AttributeError/ValueError and non-integer tallies in FloorError naming the instrument; _spawn sets returncode on the timeout path and reads stdout/stderr as UTF-8 with errors='replace'. main reports it as 'floor scoreboard refused' (tested end to end)."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. floor.py:137, 140, 338-345 at 1fa2309. Empty stdout raises IndexError, a missing field KeyError, a stray line JSONDecodeError -- tracebacks rather than the 'refused' verdict main() promises. The timeout path never sets Popen.returncode (ResourceWarning), and stdout/stderr are read with the locale encoding. Fix: wrap parse failures in FloorError, set returncode on kill, read output as UTF-8 with errors='replace'.
