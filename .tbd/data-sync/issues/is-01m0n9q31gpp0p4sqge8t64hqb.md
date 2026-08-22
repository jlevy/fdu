---
type: is
id: is-01m0n9q31gpp0p4sqge8t64hqb
title: Report.render omits the CLI's display notes
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:53:52.175Z
updated_at: 2026-08-22T17:53:52.175Z
---
Three parity sessions differ only by a missing note line, e.g.

  note: --analyze lines,code,words read 176 B; no selected view displays content metrics — try --view families, languages, or all

The CLI prints these; Report.render does not. They are not decoration: this one tells a user their analysis work produced nothing visible, which is the exact defect the content axis was introduced to prevent. A Python caller rendering a report gets no such warning.

Either the notes belong in the rendered output (so render() is genuinely what the CLI writes) or they belong on the Report as data the caller can inspect. Probably both -- data on the value, included when rendering text.
