---
type: is
id: is-01m3rg11pvht69maskknprscpw
title: "Atomic writes: benchmark harness (explorations/benchmarks)"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T06:28:58.715Z
updated_at: 2026-09-30T07:13:13.032Z
closed_at: 2026-09-30T07:13:13.032Z
close_reason: "explorations/benchmarks writes through explorations/benchmarks/atomic_write.py, a byte-identical copy of scripts/atomic_write.py enforced by the check: realtree __main__ (baseline, run, render, capture), compare_tools, floor, installed_command, provenance, subjects, record, ledger, summary, timeline, report_html; corpus.py markers/manifests, runner.py results and stdout artifacts (exclusive 'x' kept), report.py, run.py/generate.py (json.dumps), spikes/code_analysis_pair.py. Gzipped evidence: new ledger.store + 'python -m benchmarks.realtree store' + 'make perf-store' write run.json.gz deterministically (filename='', mtime=0, level 9) via open_atomic; all 16 committed run.json.gz recompress byte-identically; guide updated. Listed exceptions: corpus scan-tree writes and in-place mutations, corpus_cache materialization, gen_tree.py, planted corrupt snapshot, child captures in scratch, drop_caches. Evidence: make perf-test 395 OK, perf-evidence-check, perf-ledger-check, perf-report-check pass; make perf-report and perf-ledger regenerate byte-identical (no git diff); harness suite 70 tests pass except test_probe (needs a perf_probe cargo build, not run here). Commits f7966a8f, acb2882b."
resolution: null
duplicate_of: null
---
Run artifacts, records, ledger, report, timeline, fingerprints, corpus cache and gzipped evidence are written atomically.
