---
type: is
id: is-01m3rg116mr9nvstgsfjq7ma4w
title: "Atomic writes: gate, QA and Node runners"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:58.196Z
updated_at: 2026-09-30T06:28:58.196Z
---
scripts/run_installed_cli_qa.py, scripts/qa_peer_agreement.py, scripts/check-yaml.mjs, scripts/run-parity.mjs and tests/golden/bin/*.mjs write outputs through an atomic helper (test inputs such as planted cache fixtures excepted, with the reason stated).
