---
type: is
id: is-01m0y1skdmrymm5zphnstcdn86
title: Measure final performance, dependency, and size acceptance
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nshe4qenm5s8ce206xa3k
  - is-01m10nshrq8ska2thptbjmp8vs
  - is-01m10nsj426pyks0x8h9azvfka
created_at: 2026-08-26T03:28:35.764Z
updated_at: 2026-08-28T00:15:25.065Z
---
Measure cold usefulness and completion, settled query and continuation work, change latency, CPU, memory, dependency trees, CLI binary size, wheel size, and GIL boundary cost on the same corpus. Publish exact revisions and regimes and record the explicit rollback/default-provider decision without changing defaults in this bead.

## Notes

## Measured baseline recorded 2026-08-27 at `328ca65`

Partial evidence toward this bead's size and dependency acceptance. Not acceptance
itself: no wheel size, startup, memory, or GIL boundary cost is measured yet, and these
are macOS numbers from one host.

Command-line binary, shipped release profile (`lto = true`, `codegen-units = 1`,
`strip = true`), built `--locked --release -p fdu --all-features`:

| Build | Raw bytes | Gzip bytes |
| --- | --- | --- |
| `main` | 2,523,264 | 1,115,576 |
| `27aeed0` | 2,820,896 | 1,252,875 |
| `328ca65` | 2,820,896 | 1,254,332 |

Two findings:

1. The File Rollup registry parser and `EntrySelection` cost the command line zero raw
   bytes. `27aeed0` and `328ca65` are byte-identical because full LTO eliminates a parser
   the command line never calls; it uses the compiled registry.
2. **Open question for this bead.** The rewrite carries +297,632 raw bytes (+11.8%) and
   +137,299 gzip (+12.3%) over `main`, all committed before `328ca65`. Full LTO with
   `codegen-units = 1` means this is reachable code, not dead weight, in a binary that
   never opens a root. Attribute it per module and either justify it against the Phase 3A
   budget or reduce it. The plan treats unexplained growth here as blocking.

Dependency evidence at `328ca65`: `cargo tree -p fdu --edges normal` contains no pyo3,
tokio, async-std, reqwest, hyper, or axum; 39 dependencies total.

Command-line non-regression evidence at `328ca65`:

- the `fdu` crate has zero source changes against `main`; only `Cargo.toml` moves, adding
  `gitignore` to default features
- one golden differs from `main` (`cli-content.tryscript.md`): the
  `type_rules_fingerprint` value moved because the registry changed, and several cases
  replaced `node -e` parser wrappers with direct invocations under the golden
  observability rule
- `gitignore` defaulting on does not change the answer; A/B against a pre-rewrite binary
  on a tree with `build/` and `*.log` ignored produced byte-identical output
- `--help` differs by one word: `-d, --depth` reads `[tree default: 2]`

Still owed by this bead: wheel and binding bytes, cold startup, one-shot scan time, peak
memory, change latency, GIL boundary cost, and the paired protocol under `fdu-giss`.
