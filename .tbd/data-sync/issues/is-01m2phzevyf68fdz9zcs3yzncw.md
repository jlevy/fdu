---
type: is
id: is-01m2phzevyf68fdz9zcs3yzncw
title: Install the verified release candidate globally as a uv tool for manual testing
kind: chore
status: open
priority: 2
version: 1
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:53.117Z
updated_at: 2026-09-17T02:08:53.117Z
---
Install the CI-built wheel with `uv tool install --python 3.14 <wheel>` (the default uv interpreter on
this host is free-threaded 3.14t, which cannot load abi3 wheels). ~/.cargo/bin/fdu
(0.1.0-dev+gb75bf85a3, 2026-08-30) precedes ~/.local/bin on PATH and would shadow the tool; remove or
rename it with the maintainer's agreement. The maintainer's ~/.config/uv/uv.toml excludes packages newer
than 7 days and does not exempt fdu, so plain `uvx fdu` will not see the published release for a week.
