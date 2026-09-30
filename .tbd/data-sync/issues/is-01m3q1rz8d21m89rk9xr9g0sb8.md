---
type: is
id: is-01m3q1rz8d21m89rk9xr9g0sb8
title: Lint and test the Linux native reader on aarch64
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T17:00:39.564Z
updated_at: 2026-09-30T10:33:46.960Z
closed_at: 2026-09-30T10:33:46.960Z
close_reason: "Fixed in 9a43b6cf (ci: lint and test the engine on Linux arm64), on #164 (claude/stability-fixes). The new 'Engine on Linux arm64' job passed on the PR head df191d14 (CI run 36702345178, job 109844663398): clippy and the engine tests, including the Linux native reader, on aarch64."
resolution: null
duplicate_of: null
---
The manylinux aarch64 wheel ships H169's unsafe getdents64/statx reader, which has never executed: CI tests x86_64 only, the aarch64 wheel skips its smoke test (release.yml), and make cross-lint lacks the target. Add aarch64-unknown-linux-gnu to CROSS_TARGETS and run the linux_dents tests on an arm64 runner before 0.2.2.

## Notes

2026-09-30, claude/stability-tooling 9a43b6cf: CI job `test-linux-arm64` ("Engine on Linux arm64", ubuntu-24.04-arm) runs `cargo clippy --locked -p fdu-core --all-targets --all-features -- -D warnings` and `cargo test --locked -p fdu-core --all-features`; `make cross-lint` now includes aarch64-unknown-linux-gnu. Close this bead when that job first passes on the pull request: until then nothing has executed the reader on arm64.

aarch64 review of `crates/fdu-core/src/scan/linux_dents.rs` (read, not yet executed):
- `statx` layout: `libc::statx` is the generic glibc definition on every Linux arch, and the compile-time `size_of::<libc::statx>() == 256` assertion is now checked on aarch64 by the job. Fields read (`stx_mode` u16, `stx_size`/`stx_blocks`/`stx_ino` u64, `tv_sec` i64, `tv_nsec` u32, dev major/minor u32) have the same widths as on x86_64; `makedev` yields a u64 `dev_t` on both.
- `linux_dirent64`: arch-independent (u64, i64, u16 `d_reclen`, u8 `d_type`); the kernel pads `d_reclen` to 8 on every arch. `parse_record` reads the header byte-wise with `from_ne_bytes` and casts nothing, so alignment cannot fault; aarch64 Linux is little-endian, like the fixtures' `to_ne_bytes`. The getdents window stays 8-aligned in practice because every chunk is a sum of 8-padded records.
- `c_long`: 64-bit on aarch64, as on x86_64; `libc::syscall` returns it and the code converts with `try_from`.
- Variadic `libc::syscall` with 32-bit `c_int`/`c_uint` arguments: AAPCS64 leaves a 32-bit argument's upper register half unspecified, as SysV x86_64 does. The kernel's SYSCALL_DEFINE wrappers cast each register to the declared `int`/`unsigned int`, so `AT_FDCWD` (-100) and the flags arrive intact; std's own `syscall!` fallback passes the same types.
- Target-varying constants: `SYS_getdents64` (61) and `SYS_statx` (291) use the generic table on aarch64, and `O_DIRECTORY`/`O_NOFOLLOW` differ in value from x86_64 (0o40000/0o100000 against 0o200000/0o400000). All come from `libc`, none is written as a literal, so each is correct per target; this is the main reason the reader needs an aarch64 run rather than an x86_64 one.
- `c_char` is unsigned on aarch64; the module uses it only as a pointer cast target (`cast::<libc::c_char>()`, `ptr::null`), never as a value, so signedness cannot change behavior.
No defect found. Still open beyond this bead: the release's aarch64 wheel is cross-built and not smoke-tested (release.yml `native: false`); running that row on ubuntu-24.04-arm natively would test the shipped artifact itself.

2026-09-30, re-checked at claude/stability-tooling 6ff2d880: `make supply-chain` passes (45 tests; 85 action uses and all bootstrap pins verified), so the job needs no new pin, and no other policy or test enumerates CI jobs or runner labels. `linux_dents` is gated on `target_os = "linux"` and `target_env = "gnu"` with no `target_arch`, so the arm64 job compiles and runs it and its tests. The branch is pushed but has no pull request and no CI run yet, so nothing has executed the reader on arm64: left open until the `Engine on Linux arm64` job passes.
