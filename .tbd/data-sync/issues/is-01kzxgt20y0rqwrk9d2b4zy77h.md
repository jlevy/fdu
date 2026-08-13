---
type: is
id: is-01kzxgt20y0rqwrk9d2b4zy77h
title: Preserve cached partial diagnostics across CLI and Python
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/done/plan-2026-08-12-fdu-file-content-metrics.md
labels: []
dependencies: []
parent_id: is-01kzx1c089k0ssb8t3vy000fq9
created_at: 2026-08-13T12:16:03.101Z
updated_at: 2026-08-13T12:27:39.528Z
closed_at: 2026-08-13T12:27:39.523Z
close_reason: Rust, CLI, and Python now preserve cached partial completeness, diagnostics, and coverage; 326 Rust tests, 92 tryscript scenarios, installed-wheel smoke, and the complete make check gate pass.
---
A content sidecar can restore invalid_utf8, too_large, or unsupported records while OpenReport is correctly partial, but CLI machine errors remain empty and Python derives Index.complete from errors alone. Add end-to-end and installed-wheel regression coverage, retain a cached-partial diagnostic, and make Python completeness explicit.
