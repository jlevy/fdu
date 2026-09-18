---
type: is
id: is-01m2phzc8bg4js1chd86rs8pyj
title: "Registry pages: crates.io README links and PyPI install instructions"
kind: task
status: closed
priority: 0
version: 3
labels:
  - release
  - packaging
  - docs
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:50.442Z
updated_at: 2026-09-18T03:07:28.144Z
closed_at: 2026-09-18T03:07:28.144Z
close_reason: "Implemented on PR #87: SIGINT, registry READMEs, caret pins, version stamp/LF, 0.2 API note, 200ms interval, transient-summary --no-gitignore, python-smoke --python, JSON 2^53."
---
- The `fdu` crate uses `readme = "../../README.md"`; crates.io resolves its ~31 relative links against
  `path_in_vcs = crates/fdu`, so they 404. Give the crate its own README with absolute links.
- The PyPI page (crates/fdu-py/README.md) has no `uv tool install fdu`, `uvx fdu`, or `uv add fdu`
  line, includes contributor-only `make python-smoke`, and `[project.urls] Documentation` points at
  docs.rs (Rust docs).
- Say that wheels are abi3 for GIL-enabled CPython 3.12+: uv on this host selects free-threaded
  3.14t by default, which cannot load the wheel (`--python 3.14` works).

Being addressed by the documentation PR (branch claude/release-docs-accuracy). Changes release
artifacts, so it needs the rehearsal rerun.
