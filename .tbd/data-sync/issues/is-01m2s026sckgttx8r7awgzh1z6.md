---
type: is
id: is-01m2s026sckgttx8r7awgzh1z6
title: Python analyze signatures still hardcode none instead of NONE_LABEL
kind: task
status: closed
priority: 3
version: 3
labels: []
dependencies: []
created_at: 2026-09-18T00:53:32.076Z
updated_at: 2026-09-18T00:55:02.212Z
closed_at: 2026-09-18T00:55:02.212Z
close_reason: Three pyo3 analyze signatures now read AnalysisSet::NONE_LABEL via ANALYZE_DEFAULT, with a const assert that Request::DEFAULTS.content is empty.
---
Review S2 leftover: crates/fdu-py/src/lib.rs has three pyo3 signatures that still say analyze = "none" while read_controls and words_per_page read Request::DEFAULTS. CLI already uses AnalysisSet::NONE_LABEL plus a const assert. Change those three signatures to read the table the same way. Do not invent a new default.
