---
type: is
id: is-01m3kw44mxqjkc29qd0y4stxzj
title: "watch: Windows ReadDirectoryChangesW overflow is never signaled by notify 8.2.0"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T11:24:10.780Z
updated_at: 2026-09-28T11:24:10.780Z
---
fdu-822y review (2026-09-28): notify 8.2.0 routes ERROR_NOTIFY_ENUM_DIR into the '_' arm of handle_event (windows.rs:339-368), which logs and unwatches the directory instead of emitting Flag::Rescan. crates/fdu-core/src/watch.rs:15-17 claims Windows overruns arrive as Flag::Rescan, and the RenameReporting doc (158-160) lists Windows as EachSide with no loss caveat. Pre-existing; the old per-rename root reconciles hid it on busy trees. Correct both comments; detect overflow (upstream fix or a periodic root check on Windows).
