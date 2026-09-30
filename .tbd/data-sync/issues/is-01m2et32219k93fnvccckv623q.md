---
type: is
id: is-01m2et32219k93fnvccckv623q
title: Shared CARGO_TARGET_DIR across worktrees runs stale test binaries (mtime-only freshness)
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-14T01:56:44.224Z
updated_at: 2026-09-30T02:45:23.329Z
closed_at: 2026-09-30T02:45:23.328Z
close_reason: "Fixed in d7045b65: every Make target that compiles a workspace crate (cargo or maturin) depends on a new target-owner step that records the owning checkout in <target>/.fdu-checkout and, when another or an unrecorded checkout built there last, removes the fdu-* fingerprints so cargo rebuilds those crates (dependencies untouched); python-sdist-smoke unsets CARGO_TARGET_DIR so the sdist builds from its own contents. AGENTS.md and performance-loop.md document the guard and that bare cargo has none. scripts/cargo-target.test.mjs drives target-owner through unrecorded/same/foreign/missing directories and enforces the prerequisite on every compiling target (fails before, passes after). Not proven here: that cargo recompiles after the fingerprint removal (no builds allowed on this host) -- the coordinator's make check on a shared or pre-existing target directory should print the target-owner note and Compiling lines for fdu-core/fdu/fdu-py."
resolution: null
duplicate_of: null
---
Found by the #51/#52 fix verifier on 2026-09-13. With one CARGO_TARGET_DIR shared by several git worktrees, cargo judges workspace crates fresh by source mtime, so after checking out a different branch a test binary built by another worktree from different sources can run as-is: 'cargo test -p fdu --test detached_performance_invariants' printed no Compiling line, finished in 0.04 s, and passed from a binary lacking the code under test (ee5fe1c's twin). This invalidates any local 'passed' without a 'Compiling <crate>' line. #52's PR already noted an artifact-identity preflight catching a stale shared-target executable for timing. Mitigation used: 'find crates -name *.rs -exec touch {} +' before builds. Follow-up: document the hazard in performance-loop.md / AGENTS.md and have agent briefs and the realtree harness refuse shared-target results without a fresh-compile or artifact-identity check.
