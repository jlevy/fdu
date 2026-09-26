---
type: is
id: is-01m3fb7r2tdbfavrewz0hc7310
title: Verify fresh uvx skill and uv tool installation; clarify install docs
kind: task
status: closed
priority: 1
version: 2
labels:
  - release
dependencies: []
created_at: 2026-09-26T17:12:05.452Z
updated_at: 2026-09-26T17:38:22.589Z
closed_at: 2026-09-26T17:38:22.588Z
close_reason: Fresh published-wheel uvx and uv tool installs passed with builds disabled; clean project and user-wide skills installed and verified; public install documentation updated; make check passed.
resolution: null
duplicate_of: null
---
After the public 0.1.0 release, remove local fdu tool and skill installs, test the documented uvx zero-CLI-install skill setup and a clean uv tool install from PyPI, then update public docs to distinguish skill installation from CLI installation and show the zero-compile path for uv users. Follow the tbd agent skill distribution guidance and verify the installed skill on a clean fixture.
