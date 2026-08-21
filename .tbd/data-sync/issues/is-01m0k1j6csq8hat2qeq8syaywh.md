---
type: is
id: is-01m0k1j6csq8hat2qeq8syaywh
title: "Render --help compactly: no blank line between flags within a section"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k1hjk2w50cmaxrc3rwmvc8
created_at: 2026-08-21T20:52:54.295Z
updated_at: 2026-08-21T21:05:36.505Z
closed_at: 2026-08-21T21:05:36.504Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 129 goldens).
---
`--help` puts every flag's description on its own line and a blank line between flags,
even inside a section, which roughly doubles its height and makes a section hard to scan
as a group. `-h` is already compact:

  -h      --scan-depth <N>  Limit scanning and retention to N entry levels
  --help  --scan-depth <N>
              Limit scanning and retention to N entry levels
          <blank>

Investigate whether clap 4 can render long help compactly -- `next_line_help(false)` is
the obvious lever, and a custom `help_template` is the fallback. If clap always spaces
long help and neither works, say so and close this rather than fighting the formatter:
the alternative is to accept the height, since the guide moved to `--docs` and `-h` is
already the scannable form.
