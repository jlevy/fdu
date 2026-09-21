---
type: is
id: is-01m32wbcf7yb2afmpdrs2t3d9h
title: "PR #98 review S3: keep the Windows benchmark oracle independent of the engine's API"
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
created_at: 2026-09-21T21:01:02.823Z
updated_at: 2026-09-21T21:01:02.823Z
---
Suggestion from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. explorations/benchmarks/corpus.py:_windows_engine_attrs reads the same Windows API the same way as the engine, so the oracle is no longer independent for those six fields on Windows. Decide whether the oracle should read identity and change time through a different route (for example Python's os.stat, which uses FileIdInfo, or fsutil) so that an engine mistake cannot be mirrored by the oracle.
