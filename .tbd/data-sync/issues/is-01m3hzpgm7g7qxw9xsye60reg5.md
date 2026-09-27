---
type: is
id: is-01m3hzpgm7g7qxw9xsye60reg5
title: Install reviewed alpha stack locally for manual testing
kind: task
status: closed
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-09-27T17:48:09.723Z
updated_at: 2026-09-27T17:50:32.251Z
closed_at: 2026-09-27T17:50:32.250Z
close_reason: Built clean release wheel from48248a8a6aa0e64f2626838b560d6e828e4f5481 with fresh external Cargo target; replaced uv tool at ~/.local/bin/fdu. Verified PATH and absolute executable both report fdu0.1.0-dev+g48248a8a6 (no dirty suffix). Installed code-analysis smoke passed on golden fixtures (241 code,55 comment,48 blank; expected unsupported coverage visible). Wheel SHA256973d277c0ce4482aa37b683ea6d68e10e7ce6801807e7a579898135fa1d1ad51. Package metadata remains0.1.0; CLI identifies development commit. No source change or publication.
resolution: null
duplicate_of: null
---
Build clean wheel from48248a8a in external task target, replace current uv tool fdu0.1.0, verify version and basic code overview. No source edits or publication.
