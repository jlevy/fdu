# Installed CLI QA — 2026-09-18

Numbers from one sequential run of
[cli-installed-e2e.qa.md](../../../tests/qa/cli-installed-e2e.qa.md) via
`scripts/run_installed_cli_qa.py`. Replace this table when revising later; keep the
playbook’s procedure.

## Run Identity

| Field | Value |
| --- | --- |
| Binary | `/Users/levy/.local/bin/fdu` |
| Version | `fdu 0.1.0-dev+gcb9666a2a` |
| Host | macOS Darwin 25.5.0 |
| Cache isolation | fresh `XDG_CACHE_HOME` per arm; no `--cache-clear` on the user cache |
| Timing | `/usr/bin/time -l` wall time and peak RSS |
| Overlap | none; one fdu process at a time |

Fixture bindings for this run only:

| Variable | This run |
| --- | --- |
| `FDU_QA_SMALL` | `/Users/levy/wrk/github/fdu` (34,145 files / 1.0 GiB, including nested `.claude/worktrees`) |
| `FDU_QA_MEDIUM` | `/Users/levy/wrk/aisw/trading` (~39 GiB, 1,198,672 files) |
| `FDU_QA_MEDIUM_ANALYZE` | `/Users/levy/wrk/aisw/trading/docs` (667 files / 24 MiB) |
| `FDU_QA_LARGE` | `/Users/levy/Library` |

## Headline: Cache × Analyze (Small Tree)

Same analyzer set, isolated `XDG_CACHE_HOME`, `--cache=off` vs `--cache=auto`. Footer
`fresh` / `cached` is the reuse signal.

| Analyzer | Cache | Run | Real s | RSS MiB | Footer |
| --- | --- | --- | ---: | ---: | --- |
| code | off | 1 | 4.090 | 110.1 | 34,145 fresh, 0 cached; 373 MiB read; cold scan |
| code | off | 2 | 3.530 | 113.1 | 34,145 fresh, 0 cached; 373 MiB read; cold scan |
| lines | off | 1 | 2.730 | 112.3 | 34,145 fresh, 0 cached; 373 MiB read; cold scan |
| lines | off | 2 | 2.620 | 111.4 | 34,145 fresh, 0 cached; 373 MiB read; cold scan |
| code | auto | 1 | 3.120 | 126.2 | 34,145 fresh, 0 cached; 373 MiB read; cold scan |
| code | auto | 2 | 0.880 | 147.5 | 0 fresh, 34,145 cached / 947 MiB; 0 B read; warm revalidation |
| lines | auto | 1 | 3.120 | 130.6 | 34,145 fresh, 0 cached; 373 MiB read (sidecar replaced; see note) |
| lines | auto | 2 | 0.760 | 147.0 | 0 fresh, 34,145 cached / 947 MiB; 0 B read; warm revalidation |

`--cache-status` after the off arm: no snapshots.
After the on arm: one snapshot under the isolated dir (40,013 entries).

**Reuse:** the second `auto` run for the same `--analyze` value reused every analysis
record and dropped wall time by about 3.5× (`code` 3.12 s → 0.88 s; `lines` 3.12 s →
0.76 s). The second `off` run stayed content-cold (`0 cached`).

**Sidecar key:** `on-lines-1` ran after `on-code-2` in the same isolated dir.
The sidecar is keyed by analyzer set, so switching `code` → `lines` reread every
eligible body (`34,145 fresh`). That is the documented contract, not a cache miss on the
second `lines` run.

These analysis totals include nested worktrees and vendored files in this checkout.
They are not a crate-only SLOC figure.
From the first `auto --analyze=code` languages view: Rust 1,286,734 code lines; Python
1,123,778; JavaScript 496,123; TypeScript 357,005.

## Small-Tree Views

Metadata `--cache=auto` still reported `cold scan` on the immediate second tree view
(0.430 s → 0.300 s). That matches the usage guide: a metadata-only one-shot may skip
loading a snapshot that cannot make the work cheaper.

| Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | --- |
| help | ok | 0 | 0.060 | 22.5 |  |
| version | ok | 0 | 0.070 | 22.3 |  |
| docs | ok | 0 | 0.070 | 22.5 |  |
| skill | ok | 0 | 0.070 | 22.3 |  |
| tree-cold | ok | 0 | 0.430 | 42.3 | 34,145 files / 1.0 GiB |
| tree-warm | ok | 0 | 0.300 | 43.8 | still `cold scan` |
| summary | ok | 0 | 0.310 | 42.7 | 34,145 files, 5,704 dirs |
| languages | ok | 0 | 0.330 | 49.8 |  |
| families | ok | 0 | 0.320 | 47.8 |  |
| types | ok | 0 | 0.370 | 47.9 |  |
| extensions | ok | 0 | 0.250 | 40.8 |  |
| documents-no-analyze | ok | 2 | 0.060 | 22.4 | usage: view requires `--analyze` |
| recent | ok | 0 | 0.260 | 48.4 | `--limit=10` |
| largest | ok | 0 | 0.630 | 53.3 |  |
| files | ok | 0 | 0.670 | 50.2 | `--limit=10` |
| full | ok | 0 | 0.460 | 51.4 |  |
| combo-kinds | ok | 0 | 0.320 | 47.2 | families,types,extensions |
| exclude-ignored-summary | ok | 0 | 0.280 | 45.6 | 326 MiB / 15,842 files (default 1.0 GiB / 34,145) |
| depth-limit-tree | ok | 0 | 0.290 | 43.6 |  |
| scan-depth-1-summary | ok | 0 | 0.080 | 23.2 |  |
| json-summary | ok | 0 | 0.290 | 42.5 | schema `fdu.report/5` |
| yaml-summary | ok | 0 | 0.280 | 42.3 |  |
| analyze-words | ok | 0 | 4.200 | 136.2 | analyzer-set change; fresh reread |
| analyze-all | ok | 0 | 3.330 | 139.2 | same |
| json-analyze-code | ok | 0 | 3.020 | 135.1 | schema `fdu.report/6`; `physical_lines` filled |
| yaml-analyze-lines | ok | 0 | 2.960 | 125.4 |  |
| text-analyze-code-summary | ok | 0 | 2.860 | 126.6 | note: summary does not display content metrics |
| watch-sigint | ok | -2 | 3.015 |  | SIGINT after one file create; exited |

## Medium Tree

| Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | --- |
| med-tree-cold | ok | 0 | 16.560 | 766.3 | 1,198,672 files / 39 GiB |
| med-tree-warm | ok | 0 | 19.840 | 794.2 | still `cold scan`; tree was also mutating |
| med-summary | ok | 0 | 19.360 | 816.8 |  |
| med-languages | ok | 0 | 19.200 | 1136.3 | peak RSS this phase |
| med-combo-kinds | ok | 0 | 21.780 | 999.2 |  |
| med-recent | ok | 0 | 20.880 | 1118.4 | `--limit=10` |
| med-json-summary | ok | 0 | 19.860 | 700.7 | complete, 0 errors |
| med-analyze-code-subdir | ok | 0 | 0.210 | 26.5 | `docs/` only; 667 fresh |
| med-analyze-code-subdir-warm | ok | 0 | 0.070 | 25.7 | 667 cached; 0 B read |

`--analyze` was not run on the whole medium tree.
The `docs/` languages view printed only the “Percentage column: code lines” header: that
subtree has no programming-language rows under `--analyze=code`.

The second whole-tree metadata run was slower, not faster.
The footer remained `cold scan`, and tbd sync files in that tree were rewritten during
the window (recent view).
Do not treat 16.6 s vs 19.8 s as a cache regression without a quiet tree.

## Bounded Library

| Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | --- |
| large-summary-depth1 | ok | 0 | 0.080 | 23.4 | 6 files / 149 dirs at `--scan-depth=1` |
| large-summary-depth2 | ok | 2 | 0.230 | 26.8 | partial; 26 TCC `Operation not permitted` warnings |
| large-tree-preferences | ok | 0 | 0.080 | 25.1 | `--scan-depth=2 --depth=1 --limit=10` |
| large-tree-logs | ok | 0 | 0.080 | 23.7 | same bounds |

No `--analyze` on the Library root.
No SIGKILL, no RSS climb, no 180 s hang.
TCC at depth 2 was a documented partial (exit 2) with one warning per denied directory,
not a crash. This does not close the unbounded-Library SIGKILL risk; it only shows the
bounded escalate completed.

## Findings

- **Not a product bug:** `--view=documents` without `--analyze` exits 2 and prints that
  views never enable analysis.
  The harness now expects that status.
- **Cache off vs on works** for a repeated identical `--analyze` on this tree.
- **Metadata one-shots stay `cold scan`** under `--cache=auto`, as documented.
- **No new Library-scale bead.** Bounded commands did not reproduce SIGKILL.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
