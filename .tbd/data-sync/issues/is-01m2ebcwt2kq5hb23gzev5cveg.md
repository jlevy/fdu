---
type: is
id: is-01m2ebcwt2kq5hb23gzev5cveg
title: "PR #48 review PY-1: a poisoned index surfaces as a bare RuntimeError in Python"
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2ebb348tnqdeqn4fddykv4s
created_at: 2026-09-13T21:39:57.889Z
updated_at: 2026-09-13T21:39:57.889Z
---
Medium. crates/fdu-py/src/opened_binding.rs:69 -> lib.rs:61; python/fdu/opened.py:1164-1184. opened_py_err maps lifecycle and journal poison to OpenedIndexError but not Error::IndexLockPoisoned, which becomes a bare RuntimeError that _opened_call does not catch; a Rust panic in refresh() surfaces as pyo3 PanicException (a BaseException), undocumented. Fix: map IndexLockPoisoned to OpenedIndexError, add a final RuntimeError arm to _opened_call mirroring _call, document PanicException. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5192314101 (head c853f7c).
