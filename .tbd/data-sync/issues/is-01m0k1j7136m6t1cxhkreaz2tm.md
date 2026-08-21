---
type: is
id: is-01m0k1j7136m6t1cxhkreaz2tm
title: Wrap help at min(terminal width, 160), which needs clap's wrap_help feature
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k1hjk2w50cmaxrc3rwmvc8
created_at: 2026-08-21T20:52:54.946Z
updated_at: 2026-08-21T21:05:36.829Z
closed_at: 2026-08-21T21:05:36.827Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
Wrap help at the terminal width, capped at 160 columns -- whichever is smaller -- so long
descriptions stay readable on a wide terminal without running to the full width of an
ultrawide one.

clap has exactly this: `Command::max_term_width(160)` takes the minimum of the detected
terminal width and the cap. Detection needs clap's `wrap_help` feature, which is not
enabled here (only `derive` is), and which pulls in `terminal_size`.

That makes it a dependency change, so it goes through SUPPLY-CHAIN-SECURITY.md and the
14-day cool-off before anything else. Check what `terminal_size` costs -- it is small and
widely used, but "small and widely used" is the argument every dependency makes, and this
crate's always-on list is deliberately short.

If the dependency is not wanted, `max_term_width` still caps at 160 without detection;
what is lost is narrowing to an actually-small terminal.
