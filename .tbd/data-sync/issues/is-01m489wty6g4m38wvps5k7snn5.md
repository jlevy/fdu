---
type: is
id: is-01m489wty6g4m38wvps5k7snn5
title: Pluralize the code-coverage notes ("1 files with unclassified type")
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-06T09:49:40.148Z
updated_at: 2026-10-06T09:49:40.148Z
---
crates/fdu-core/src/report_format/report_epilogue.rs code_coverage(): 'note: {n} files with unclassified type' and 'note: {n} source files with unknown ignore classification' do not pluralize (prints '1 files'). Use plural() like the 'languages analyzed' note. Found by the skill review (fdu-dynr).
