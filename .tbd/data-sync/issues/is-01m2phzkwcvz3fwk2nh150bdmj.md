---
type: is
id: is-01m2phzkwcvz3fwk2nh150bdmj
title: Tag v0.1.0 (SSH-signed) on the rehearsed release commit
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzm7r5vw97jr9y4mtzxxs
parent_id: is-01kzg4c6vnh98mqrpkzw7ydne0
created_at: 2026-09-17T02:08:58.251Z
updated_at: 2026-09-26T17:25:48.178Z
closed_at: 2026-09-26T17:25:48.176Z
close_reason: Signed v0.1.0 tag pushed and verified on release commit 7cf7f1b4b39930ebcf54df7ebd2645d995e4e458; release plan and checkout validated.
resolution: null
duplicate_of: null
---
Record the rehearsal run's headSha as the release commit; recheck that fdu-core, fdu and PyPI fdu
return 404 before pushing; `git tag -s v0.1.0`, `git tag -v v0.1.0 && git push origin v0.1.0`; clone the
tag and run `resolve_plan.py --mode release --validate-checkout`; GitHub shows the tag Verified.
