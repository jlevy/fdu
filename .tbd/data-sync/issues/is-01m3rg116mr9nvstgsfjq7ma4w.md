---
type: is
id: is-01m3rg116mr9nvstgsfjq7ma4w
title: "Atomic writes: gate, QA and Node runners"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:58.196Z
updated_at: 2026-09-30T06:48:59.457Z
closed_at: 2026-09-30T06:48:59.457Z
close_reason: "Converted to scripts/atomic_write.py / scripts/atomic-write.mjs: run_installed_cli_qa.py results.tsv/md/json (its local mkstemp writer removed), qa_peer_agreement.py --json, run-parity.mjs artifact and per-run corpus, tests/path_independence runner.py diffs and --record, registry.py judged runs and merged registry. Listed in scripts/check-atomic-writes.mjs as test inputs: check-yaml.mjs scan tree, tests/golden/bin/* (planted cache fixtures, watch changes), qa_peer_agreement self-test tree, QA watch tree, path_independence matrix.py mutations (in-place rewrite is the change under test) and the warmed-cache copytree; QA per-command stdout/stderr stay incremental (only the same run reads them, after the child exits). make release-test 214 OK, make test-path-independence 37 OK, ruff clean, node --check run-parity.mjs. Commit 227963b5."
resolution: null
duplicate_of: null
---
scripts/run_installed_cli_qa.py, scripts/qa_peer_agreement.py, scripts/check-yaml.mjs, scripts/run-parity.mjs and tests/golden/bin/*.mjs write outputs through an atomic helper (test inputs such as planted cache fixtures excepted, with the reason stated).
