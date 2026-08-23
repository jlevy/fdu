---
type: is
id: is-01m0nzy19r3f8gg1rwz29bc5zh
title: "PR #42 review S1: report_format::view_label is a pure one-line delegation"
kind: chore
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:08.439Z
updated_at: 2026-08-23T00:39:59.393Z
closed_at: 2026-08-23T00:39:59.393Z
close_reason: "Fixed, and it turned up a third copy: crates/fdu-py/src/lib.rs had its own ten-arm view_label, identical on main. Both the wrapper and the binding's copy are gone; every site calls ViewSpec::label()."
---
crates/fdu-core/src/report_format.rs:1143. Non-blocking suggestion: its three internal call sites could use view.label() directly.
