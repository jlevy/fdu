---
type: is
id: is-01m3neefnak4j3hwn82k91d3z2
title: run_installed_cli_qa.py reads GNU time's exit status as wall time after a non-zero exit
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T02:03:38.537Z
updated_at: 2026-09-30T02:52:13.193Z
closed_at: 2026-09-30T02:52:13.192Z
close_reason: "Fixed in e08b340e: parse_time_file's BSD patterns ('<n> real', '<n> maximum resident set size') are now anchored to one line, so GNU's 'Command exited with non-zero status 2' line no longer reads as 2 s of wall time. New tests/release/test_installed_cli_qa.py (GNU clean/non-zero/signal, BSD, empty) fails before (2.0 != 0.02; 9.0 for a signal), passes after; make release-test green. No GNU time on this host, so verified against its documented output format."
resolution: null
duplicate_of: null
---
After 'Command exited with non-zero status 2', the harness parses 2 as the wall time (documents shows 2.000 s for a 0.02 s run). Parse the elapsed field by label. Found in the 0.2.1 stability pass.
