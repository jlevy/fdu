---
type: is
id: is-01kztvyt9245vp768sgav6dxrx
title: Test root-dirfd-relative directory opens (H2/H24)
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzpvt1vamkqp8fffnpwhd93v
created_at: 2026-08-12T11:33:10.049Z
updated_at: 2026-08-12T11:49:22.088Z
closed_at: 2026-08-12T11:49:22.087Z
close_reason: "Rejected and recorded as exp-024: 720k cold-index wall was neutral at -0.07% [-4.06%, +1.53%], both indexed and producer system CPU were neutral, and the producer-only wall signal did not reproduce consistently. Candidate reverted."
---
Post-exp-022 profile leaves directory open at 33.86% of cold-index samples. Test the smallest composable H2/H24 step inside the audited macOS boundary: retain one root directory fd per worker and open claimed relative paths with openat, avoiding repeated resolution of the absolute root prefix while preserving complete-directory portable fallback. Pre-registered signal: cold-scan-index and cold-scan-producer system CPU and wall, with the largest effect expected on the deep 720k cache-pressure tree. Accept only at least 3% paired wall improvement with oracle parity and no material small-tree/resource regression; otherwise record and revert.
