---
type: is
id: is-01m2pmr9n3mq4nb2r1pc328qpz
title: "Phase 1 item 1: path-independence harness with a known-violation registry"
kind: epic
status: closed
priority: 0
version: 15
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - testing
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pydzqqs5qf4n3bxtaf3twt
  - is-01m2pye0422qnb6171mmxabt4x
  - is-01m2pye0e9mmthdmnttkmbfpgq
  - is-01m2pye0rg6x3k2bzbqgvv6nrz
  - is-01m2pye12ype0stny4rb63rh5t
  - is-01m2pye1dfapb52r07ztah5nap
  - is-01m2q2zbmwp3dd406y9dwbm3q5
created_at: 2026-09-17T02:57:24.130Z
updated_at: 2026-09-17T23:46:47.556Z
closed_at: 2026-09-17T23:46:47.556Z
close_reason: "Phase 1 item 1 (path-independence harness) shipped in PR #79 and landed on main via stack merge 98379c76."
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 1: The Path-Independence Harness", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P1.1.1 to P1.1.6; their blockers carry the ordering, so this bead only groups them and closes when they do.

The harness is Python `unittest` using only the standard library, run through uv’s
Python 3.12 as `release-test` is (`Makefile:380-381`). The invariant spans the command
line and the Python package, which a Rust test cannot reach, and tryscript compares
bytes rather than parsed answers.
The case against is speed (one subprocess per case) and the installed wheel the Python
half needs; model unit tests stay in Rust.

| File | Function or type | Change |
| --- | --- | --- |
| `tests/path_independence/fixture.py` | `build_fixture(root) -> FixtureFacts` | Port `explorations/path-independence/make_fixture.sh`: seeded bytes instead of `/dev/urandom`, `os.utime` instead of `touch -t`, symlinks skipped and recorded where `os.symlink` fails |
| `tests/path_independence/matrix.py` | `REQUESTS`, `WARMERS`, `MUTATIONS`, `ROUTES`, `SUBSET` | Move the exploration’s `R`, `W`, `mutate`, and `MUTATIONS`; add the `unreadable` mutation (`chmod 000` on `src/nested`, restored in `finally`, skipped where permission bits are not enforced); routes `cli-report`, `py-report`, `py-open`, `py-scan`, and later `cli-watch-initial` |
| `tests/path_independence/runner.py` | `run_cli`, `run_py`, `normalize`, `compare -> Verdict`, `case_key` | Move the exploration’s runner; `normalize` drops only `source`, `freshness`, `scan_started_at`, and `generated_at`; verdicts are `same`, `differs`, or `outcome_class` |
| `tests/path_independence/pyrun.py` | `main` | Move; refuse to run when `fdu` imports from `crates/fdu-py/python`, the parity safety property |
| `tests/path_independence/registry.py` | `load`, `verify`, `record` | Add |
| `tests/path_independence/known-violations.toml` | registry | Add, seeded from a full Linux run |
| `tests/path_independence/test_harness.py` | comparator and registry unit tests | Add; needs no fdu build |
| `tests/path_independence/test_path_independence.py` | `PathIndependence.test_matrix` | Add; `FDU_PI_TIER=subset\|full`, `FDU_PI_SURFACES=cli\|cli,python` |
| `Makefile` | `path-independence`, `test-path-independence`, `path-independence-full`, `path-independence-record`; `check` (`:113`), `UV_BACKED_TARGETS` (`:163`), `PYTHON_LINT_PATHS` (`:345`) | Add the targets; `check` runs the subset after `parity-check` with `FDU_PYTHON=$(SMOKE_PYTHON)` |
| `.github/workflows/ci.yml` | `test` job (`:61-111`), `parity` job (`:228-274`) | Add the pinned `setup-uv` step to the `test` job, which has none, and run the command-line subset on three platforms through the Make target, so the harness runs on uv’s Python 3.12 (`tomllib` needs 3.11); the two-surface subset runs in the parity job with `.venv-parity` |
| `.github/workflows/path-independence.yml` | full matrix | Add: schedule, `workflow_dispatch`, and the `path-independence-full` label; three platforms; failing diffs uploaded; toolchain and uv pins inventoried in `supply-chain-policy.json` |
| `explorations/path-independence/` | scripts | Delete once moved; keep `results/` and a README pointing to `tests/path_independence` |

`FDU_BIN` is an absolute path, defaulting to `target/debug/fdu` from `make build` and
never resolved through `PATH`. Each invocation gets its own `XDG_CACHE_HOME`, which
`user_cache_dir` honors on every platform (`lib.rs:794-799`).

The registry is TOML, read with `tomllib` and reviewed like a golden:

```toml
[classes.content-containment]
clears_with = "Phase 1 item 2: content identity and equality serve"
bead = "fdu-gija"

[[violation]]
key = "warm/cli-report/auto/W_all/-/a_lines"
class = "content-containment"
paths = ["analysis.analyze[]", "reports[].metrics.total.metrics.physical_lines"]
```

A run fails on an unregistered difference, a registered key whose generalized paths
changed, a registered key that now matches cold and the other routes, a class with no
entries, an entry still marked `unclassified`, or a run with zero cases or zero
parseable cold answers.
An optional `platforms` field records a genuinely platform-specific entry, which is
itself a finding to explain.

The subset is 16 requests (`default`, `nogi`, `budget1k`, `scandepth1`, `exclign`,
`onlyign`, `v_summary`, `v_summary_nogi`, `v_types`, `a_lines`, `a_code`, `a_words`,
`a_all`, `a_lines_v_documents`, `a_code_langs_name_lim1`, `a_all_nogi`) across five
warmers and three policies on `cli-report`, five mutations after two warmers, and the
Python routes after two warmers: about 1,200 invocations, budgeted at 90 seconds on
Linux and 4 minutes on Windows, where process creation is slower.
The full matrix is about 20,000 invocations, budgeted at 30 minutes per platform.

The seed classes are `content-containment`, `mixed-records`, `projection-route`, and
`unverified-subtree`, each naming the item that clears it.

**Commits:**
1. Move the fixture, comparator, and matrix with `test_harness.py`; add lint paths.
2. Add `registry.py`, its tests, and `--record`.
3. Seed the registry from a full Linux run; add the Make targets and the subset in
   `make check`.
4. Add the `unreadable` mutation and its entries.
5. Add the CI steps and the full-matrix workflow with its supply-chain inventory.
6. Retire the exploration scripts and update links.

**Risks:** the Python routes use a release wheel while the command line uses a debug
build, so a stale `.venv-parity` goes undetected, as parity already accepts;
`samesize_keepmtime` may differ on Windows, where ctime is creation time; a third
workflow must pass `validateWorkflowSecurity` in `scripts/check-supply-chain.mjs`.

## Original scope (before the implementation detail)

Bring the 2026-09-17 empirical matrix into the repository (local copy: attic/path-independence-matrix-2026-09-17):
requests across every scope flag, selection filter, view, analyzer set, and size metric; warming histories;
auto, read-only, only; file mutations; command line, Python one-shot, Python Index; parsed comparison with cold
answers apart from provenance. Add a metric-independence test (a metric's cold value is identical under every
analyzer set containing its analyzer) and a writer-equality test (JSON, JSONL, YAML strict 1.1 and 1.2, Python
models; adversarial names and non-UTF-8 paths). A registry of known violations fails on any new difference and on
any stale entry, like the parity artifact. Fast subset in make check; full matrix in CI on three platforms.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 1. Harness committed at explorations/path-independence (77da71bd); move it under tests/ with a registry seeded from the warm, mutation, and cross-surface cases; add an unreadable-subtree mutation; a bounded subset runs in make check and on every PR, the full matrix on a schedule or by label; compare content and tree status (complete, errors, coverage), excluding only provenance; compare outcome classes across routes and surfaces.
