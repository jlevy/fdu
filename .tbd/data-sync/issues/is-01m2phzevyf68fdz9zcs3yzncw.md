---
type: is
id: is-01m2phzevyf68fdz9zcs3yzncw
title: Install the verified release candidate globally as a uv tool for manual testing
kind: chore
status: closed
priority: 2
version: 3
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:53.117Z
updated_at: 2026-09-26T17:26:08.377Z
closed_at: 2026-09-26T17:26:08.375Z
close_reason: Published fdu 0.1.0 wheel installed globally with uv tool install --no-build --python 3.12; fdu --version and a real JSON report passed.
resolution: null
duplicate_of: null
---
Install the CI-built wheel with `uv tool install --python 3.14 <wheel>` (the default uv interpreter on
this host is free-threaded 3.14t, which cannot load abi3 wheels). ~/.cargo/bin/fdu
(0.1.0-dev+gb75bf85a3, 2026-08-30) precedes ~/.local/bin on PATH and would shadow the tool; remove or
rename it with the maintainer's agreement. The maintainer's ~/.config/uv/uv.toml excludes packages newer
than 7 days and does not exempt fdu, so plain `uvx fdu` will not see the published release for a week.

## Notes

2026-09-23: replaced the global uv tool with a development build of main 0059ddd5 (fdu 0.1.0-dev+g0059ddd57, wheel built with maturin --release, Python 3.12). Reinstall from the release candidate when one is cut.
