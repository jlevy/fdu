---
type: is
id: is-01m2ebcdxhc9j53kv6y1pg7sqn
title: "PR #49 review FLOOR-11: probe path ignores CARGO_TARGET_DIR; build dir in world-writable /tmp"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels: []
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-13T21:39:42.640Z
updated_at: 2026-09-13T21:58:35.380Z
closed_at: 2026-09-13T21:58:35.379Z
close_reason: "Fixed: PERF_TARGET_DIR asks cargo metadata for the target directory and PERF_RELEASE/PERF_PROFILING sit under it (verified with CARGO_TARGET_DIR in make -n); --build-dir defaults to fdu-floor under cargo's target directory instead of /tmp/fdu-floor/bin."
resolution: null
duplicate_of: null
---
PR #49, review 5192251516, Low. floor.py:272, 713 at 1fa2309. The probe path assumes PROJECT_ROOT/target, ignoring CARGO_TARGET_DIR and build.target-dir (a stale binary there would be scored silently), and --build-dir defaults to the predictable /tmp/fdu-floor/bin, in a world-writable directory, for binaries the harness then executes. Fix: resolve cargo's target directory with cargo metadata -- in the Makefile's probe paths, which perf-floor now passes (FLOOR-5), and for the build-dir default, which moves under the target directory.
