---
type: is
id: is-01m3kapbv73ec6rcnpefz7wc6f
title: Isolate FDU_CACHE_DIR in Python, terminal and golden tests
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T06:19:33.607Z
updated_at: 2026-09-28T06:19:33.607Z
---
FDU_CACHE_DIR outranks XDG_CACHE_HOME. #139 made the Rust watch tests remove it, but crates/fdu-py/tests/test_directory_queries.py, test_readme_examples.py, tests/terminal/test_progress_pty.py and the tryscript goldens still set only XDG_CACHE_HOME, so a developer with FDU_CACHE_DIR exported runs them against the real cache. An empty FDU_CACHE_DIR is ignored (nonempty_env), so goldens can set it to empty. Found in the #139 senior review.
