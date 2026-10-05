---
type: is
id: is-01m44py3xgfe4rbzp85g0cr06w
title: "cli-animate: block-glyph seams with Planetaire Mono in the DOM renderer"
kind: task
status: open
priority: 2
version: 2
spec_path: packages/cli-animate/docs/project/specs/active/plan-2026-10-04-cli-animate.md
labels: []
dependencies: []
parent_id: is-01m44jm7xwyb4m5vqpf3ethg4w
created_at: 2026-10-05T00:20:35.888Z
updated_at: 2026-10-05T01:44:12.469Z
---
Planetaire's U+2588 full block stops just short of its advance, so adjacent cells show faint anti-aliased seams in browsers (xterm.js DOM renderer, asciinema-player). Options: full-advance block/shade glyphs upstream in jlevy/planetaire, or a working WebGL capture (xterm.js draws block elements itself there; headless SwiftShader screenshots came back blank).
