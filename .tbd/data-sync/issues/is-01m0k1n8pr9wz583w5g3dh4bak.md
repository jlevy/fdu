---
type: is
id: is-01m0k1n8pr9wz583w5g3dh4bak
title: One header and color system across report, --docs, and --help
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k1hjk2w50cmaxrc3rwmvc8
created_at: 2026-08-21T20:54:34.967Z
updated_at: 2026-08-21T21:05:37.147Z
closed_at: 2026-08-21T21:05:37.145Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
Three surfaces print headers and none of them agree.

  report views   ALL CAPS, styled (STYLE_VIEW_HEADER)
  --docs         ALL CAPS, unstyled -- written straight to stdout
  --help         Title case with a colon ("Scope:", "Selection:"), styled cyan bold
                 through CLI_STYLES

Pick one system and apply it everywhere: ALL CAPS headers, one style constant, colored
only when the destination is a live terminal. The style constants already exist
(STYLE_HEADING, STYLE_VIEW_HEADER, STYLE_PERFORMANCE, ...) and `paint()` already gates on
`stdout_is_terminal`, so this is unification rather than new machinery.

Constraints that must survive:
- NO_COLOR and --color never|always|auto keep working; goldens run with NO_COLOR=1
- machine formats are never styled
- the wire label stays lowercase; the uppercase header is a display concern, which
  report_format.rs already notes at the view_label definition
