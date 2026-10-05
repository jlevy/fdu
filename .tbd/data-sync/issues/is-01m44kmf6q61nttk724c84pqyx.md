---
type: is
id: is-01m44kmf6q61nttk724c84pqyx
title: "cli-animate follow-up: faster capture (CDP screenshots with optimizeForSpeed) and send each distinct frame to ffmpeg once"
kind: task
status: open
priority: 3
version: 3
spec_path: packages/cli-animate/docs/project/specs/active/plan-2026-10-04-cli-animate.md
labels: []
dependencies: []
parent_id: is-01m44jm7xwyb4m5vqpf3ethg4w
created_at: 2026-10-04T23:22:54.038Z
updated_at: 2026-10-05T01:44:12.469Z
---
Research: beginFrame works only in chrome-headless-shell and would deadlock rAF-based seeks; do not adopt unless the stage gains CSS animations. Instead use Page.captureScreenshot with optimizeForSpeed and a concat list with per-frame durations; keep the frame-exact verify passing.
