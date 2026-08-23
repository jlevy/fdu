---
type: is
id: is-01m0qs0msk75k8r89b44vqqjnz
title: "Progress mode: --progress/--progress-at on the Mode axis, sharing the watch repaint loop"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0qs19pg77zfmd3s2kg7k905
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T16:59:42.771Z
updated_at: 2026-08-23T17:00:19.769Z
---
The engine already treats scan and watch as the same thing (both are delta producers; scan takes sink: &mut dyn FnMut(Observation), reconcile takes &mut dyn FnMut(&AppliedDelta)), and the CLI already knows how to render a live feed (Cli::run_watch cli.rs:661, Cli::render_live cli.rs:851, report_format::render_change report_format.rs:1317 under STREAM_SCHEMA fdu.stream/1). So this is not a new output contract: factor run_watch's loop so watch and progress drive one renderer, and add --progress plus --progress-at <depth|entries:N|batch> to the Mode axis beside --watch. Checkpoints are LOGICAL, never intervals — wall clock is not reproducible, which is why --interval cannot serve. Each frame is an existing Report in the requested format, so every view is traceable for free. ALSO AMEND cli.rs:167: the --docs guide states 'The command never prompts, pages, or animates progress.' Nothing here animates or moves a cursor and the default stays silent, but the sentence as written forbids this, so it is amended in the same change to distinguish an animated indicator (still never) from an opt-in stream of frames.
