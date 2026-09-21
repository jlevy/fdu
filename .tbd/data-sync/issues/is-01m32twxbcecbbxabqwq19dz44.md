---
type: is
id: is-01m32twxbcecbbxabqwq19dz44
title: "PR #98 review R4: MSRV job runs on ubuntu only and cannot see the Windows-only module"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
hold: null
hold_until: null
created_at: 2026-09-21T20:35:40.012Z
updated_at: 2026-09-21T21:09:57.435Z
started_at: 2026-09-21T20:36:00.253Z
closed_at: 2026-09-21T21:09:57.435Z
close_reason: "Fixed in 650b6b08 + edd6625e on codex/release-windows-validity. The MSRV job installs x86_64-pc-windows-msvc for 1.85.0 and runs cargo check --locked --all-features --all-targets --target x86_64-pc-windows-msvc; make msrv does the same where the MSRV toolchain has the target, and AGENTS.md names the rustup +1.85.0 target add step. Found while fixing it: rust-toolchain.toml pins 1.97.1 and a directory override outranks the rustup default the toolchain action sets, so every cargo step in the MSRV job had run on 1.97.1 (rustup's note in the job log names the override) — the job checked nothing at the MSRV on any platform. RUSTUP_TOOLCHAIN=1.85.0 at job level now outranks the file. Verified: the MSRV (1.85) job passed on edd6625e with the Windows leg; locally cargo +1.85.0 check --locked --all-features --all-targets --target x86_64-pc-windows-msvc passes."
resolution: null
duplicate_of: null
---
Medium from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. .github/workflows/ci.yml:162-176 MSRV job checks ubuntu only; windows_metadata.rs is MSRV-checked by nothing required. Fix: add --target x86_64-pc-windows-msvc to the MSRV job.
