---
type: is
id: is-01m00mwx0kfer80a4h7jacdxe6
title: CI never lints platform-gated code, including the only unsafe block
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T17:25:13.875Z
updated_at: 2026-08-14T17:25:13.875Z
---
The Clippy job runs on ubuntu-latest only, so any code behind a cfg(target_os) gate is never linted - and that includes scan/macos_bulk.rs, which holds this repository's single unsafe exception and its getattrlistbulk FFI. A local make check has the same blind spot for the same reason. This was noticed while adding proc_pidinfo to perfkit: a u32-to-i32 pid cast that pedantic would flag went through both gates unremarked, and was only caught by reading the vendored libc source by hand. Compilation errors in gated code do surface, because the platform Test jobs build it, but lint findings do not, and the unsafe block is exactly where a lint is most worth having. Fix is to add macos-latest and windows-latest to the clippy job's matrix, which costs two runners on every push - or, more cheaply, to run cross-platform clippy on a schedule or only when the gated files change, since they change rarely. Worth pricing both before choosing.
