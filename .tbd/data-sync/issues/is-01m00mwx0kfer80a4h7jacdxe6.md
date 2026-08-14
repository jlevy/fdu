---
type: is
id: is-01m00mwx0kfer80a4h7jacdxe6
title: CI never lints platform-gated code, including the only unsafe block
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-14T17:25:13.875Z
updated_at: 2026-08-14T17:46:42.005Z
closed_at: 2026-08-14T17:46:42.005Z
close_reason: "Closed by make cross-lint, which runs clippy against x86_64-apple-darwin and x86_64-pc-windows-msvc locally. Checking rather than building means no cross-linker is needed, so it works anywhere, and it skips targets that are not installed rather than failing. It found four real defects on its first run, all in Windows-gated code no lint had ever seen: two uses of usize::is_multiple_of, stable since 1.87 against a declared MSRV of 1.85, which means a Windows user on the minimum could not build the crate at all - the MSRV job runs on ubuntu, where those functions do not exist. The other two were a checked cast that could be stated directly and a map().unwrap_or() on a Result. macOS came back clean, including the getattrlistbulk module holding the repository's only unsafe block, which had never been linted anywhere before this."
---
The Clippy job runs on ubuntu-latest only, so any code behind a cfg(target_os) gate is never linted - and that includes scan/macos_bulk.rs, which holds this repository's single unsafe exception and its getattrlistbulk FFI. A local make check has the same blind spot for the same reason. This was noticed while adding proc_pidinfo to perfkit: a u32-to-i32 pid cast that pedantic would flag went through both gates unremarked, and was only caught by reading the vendored libc source by hand. Compilation errors in gated code do surface, because the platform Test jobs build it, but lint findings do not, and the unsafe block is exactly where a lint is most worth having. Fix is to add macos-latest and windows-latest to the clippy job's matrix, which costs two runners on every push - or, more cheaply, to run cross-platform clippy on a schedule or only when the gated files change, since they change rarely. Worth pricing both before choosing.
