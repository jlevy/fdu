# 0.3.0 Stability Pass — 2026-09-30

This is the full record of the 0.3.0 release checklist’s stability pass (step 2 of the
[release process](../guides/release-process.md#the-steps)). The summaries lived beside
the procedures, in the
[QA playbook’s Current Status](../../../tests/qa/cli-installed-e2e.qa.md) and the
[correctness runbook’s Last Recorded Run](../guides/correctness-runbook.md#last-recorded-run),
until the [0.4.0 record](report-2026-10-09-release-0.4.0-stability-pass.md) replaced
them; they remain in those files’ history.
This report keeps every table they summarize, and says how to run the pass again.
The release engine’s performance standing is
[exp-202](../experiments/exp-202-linux-the-0-3-0-release-end-to-end-the-default-tree-48-faste.md).

## Verdict

The pass tested commit `e808f96042b9b92b722f03bd4fce8d8e2fca7efe`, tree
`df899f7da7fd2fabd908b1ec6e893f392233e273`, the head of the release pull request.
A release commit with that tree inherits every result here by tree identity; check it
with `git rev-parse <commit>^{tree}` before tagging.
Nothing failed, and no peer-agreement row is `UNEXPLAINED`.

| Gate or phase | Verdict |
| --- | --- |
| `make check` | Passed: 4,243 Rust tests, 212 goldens, parity with its 62 recorded deviations, cargo-deny and npm audit clean |
| `make cross-lint` | Passed for darwin, windows-msvc, i686, and musl; CI’s arm64 job covers aarch64 |
| `make semver-check` | Passed: each crate “starts a new compatibility series; nothing to check” |
| `make release-rehearse` | Passed: crates packaged and smoke-installed, source distribution and abi3 wheel built, artifacts inspected |
| QA phases 1–5, 7, 8 | Passed |
| QA phase 6, terminal progress | Pending, as for 0.2.1: `make test-terminal` and a 38-check pty probe passed; nobody watched a real window, and Windows has not run |
| QA phase 7, peer agreement | 12 of 12 self-test and 60 of 60 real-tree readings exact; the script exits 1 only for the known ext4 top-level directory-block difference (`fdu-83km`), each such row matched to the byte by a separate walk |
| Correctness, refusal tree as `nobody` | 23 of 23 partial and withheld; 0 mismatches |
| Correctness, served tree | 23 of 23 served `cache_only` and labelled `stale`; 0 mismatches |
| Correctness, cross-warm matrix | 30 of 30 pairs matched the cold answer; 0 violations |
| Correctness, deliberately broken | Both breaks fail as they must |

## Regime

- **Host.** A 4-vCPU Linux x86_64 virtual machine (Firecracker, kernel 6.18), on ext4,
  running as root and otherwise quiet.
- **Declared preconditions.** `FDU_TEST_ALLOW_NO_PERMISSION_BITS=1` and
  `FDU_TEST_ALLOW_NO_NATIVE_WATCH=1`, as AGENTS.md prescribes for such a host, and
  `UV_PYTHON=3.12`.
- **Candidate.** The `fdu-0.3.0-cp312-abi3-manylinux_2_34_x86_64.whl` wheel built from
  the commit, installed with `uv tool install` into an isolated tool directory;
  `fdu --version` printed `fdu 0.3.0-dev+ge808f9604`.
- **Peers.** GNU du 9.4, dust 1.2.5, pdu 0.24.0, dua 2.45.0, and diskus 0.9.0.
- **Trees.** This repository’s checkout, `~/.rustup`, `/usr`, `linux-v6.12` (a shallow
  clone of Linux v6.12), and `node-modules-dense`; `/` stands in for macOS’s `~/Library`
  as the hostile wide tree.

This pass says nothing about the macOS or Windows walk.

## Reproduce

Every step runs on the release commit in a clean worktree with its own Cargo target
directory. The gates:

```shell
git worktree add --detach "$WORKTREE" "$COMMIT"
cd "$WORKTREE"
export CARGO_TARGET_DIR="$TARGET"
export FDU_TEST_ALLOW_NO_PERMISSION_BITS=1 FDU_TEST_ALLOW_NO_NATIVE_WATCH=1  # only on a host that needs them
for gate in check cross-lint semver-check release-rehearse; do
  make "$gate" > "$LOGS/$gate.log" 2>&1; echo "make $gate -> $?"
done
```

The candidate, the QA harness, and peer agreement, with GNU `time` installed (the
harness requires `/usr/bin/time`):

```shell
# Build and install the wheel as the QA playbook's Install the Candidate describes.
uv tool install --force --python 3.12 --no-index --find-links "$WHEELS" fdu
FDU="$(command -v fdu)" FDU_QA_SMALL="$CHECKOUT" FDU_QA_MEDIUM="$LINUX_TREE" \
  FDU_QA_LARGE=/ FDU_QA_OUT="$OUT" python3 scripts/run_installed_cli_qa.py
python3 scripts/qa_peer_agreement.py   # as the playbook's Phase 7 directs
FDU_BIN="$(command -v fdu)" make test-terminal
```

The correctness runbook’s three passes, then the two breaks, which wrap the candidate so
a pass can be seen to fail:

```shell
python3 tests/correctness/build_tree.py "$TREES/tree"
setpriv --reuid=65534 --regid=65534 --clear-groups \
  env FDU_BIN="$FDU" python3 tests/correctness/warm_cold.py --refusals-only "$TREES/tree"
python3 tests/correctness/build_tree.py --without-refusals "$TREES/served"
FDU_BIN="$FDU" python3 tests/correctness/warm_cold.py "$TREES/served"
FDU_BIN="$FDU" python3 tests/correctness/cross_warm.py "$TREES/served"
# Break 1: a wrapper that turns --cache on into --cache off; both scripts must exit 1.
# Break 2: a wrapper that answers --stale-ok with a cold scan relabeled cache_only;
#          --refusals-only must exit 1.
```

Three pieces of this pass were written by hand for the run and are not yet committed:
the Phase 6 pty probe, the ext4 top-level walk that explains the peer script’s known
exit (`fdu-83km`), and the two break wrappers.
Committing them behind one entry point is the next step, so that the next pass reruns
rather than rewrites them.

## Full Tables

Paths are replaced by the tree labels above.

### Installed-CLI harness (fdu 0.3.0-dev+ge808f9604)

| Phase | Name | Verdict | Exit | Real s | RSS MiB | Note |
| --- | --- | --- | ---: | ---: | ---: | --- |
| sanity | help | ok | 0 | 0.030 | 12.3 |  |
| sanity | version | ok | 0 | 0.030 | 12.1 |  |
| sanity | docs | ok | 0 | 0.030 | 12.2 |  |
| sanity | skill | ok | 0 | 0.030 | 12.3 |  |
| views | tree-cold | ok | 0 | 0.050 | 16.5 | cold scan; fresh=0; cached=0 |
| views | tree-warm | ok | 0 | 0.060 | 16.2 | cold scan; fresh=0; cached=0 |
| views | summary | ok | 0 | 0.050 | 15.9 | cold scan; fresh=0; cached=0 |
| views | languages | ok | 0 | 0.080 | 24.4 | cold scan; fresh=0; cached=0 |
| views | families | ok | 0 | 0.110 | 23.8 | cold scan; fresh=0; cached=0 |
| views | types | ok | 0 | 0.090 | 24.4 | cold scan; fresh=0; cached=0 |
| views | extensions | ok | 0 | 0.060 | 19.8 | cold scan; fresh=0; cached=0 |
| views | documents-no-analyze | warn | 2 | 0.020 | 13.2 | empty stdout |
| views | recent | ok | 0 | 0.080 | 26.6 | cold scan; fresh=0; cached=0 |
| views | largest | ok | 0 | 0.100 | 26.4 | cold scan; fresh=0; cached=0 |
| views | files | ok | 0 | 0.200 | 35.2 | cold scan; fresh=0; cached=0 |
| views | full | ok | 0 | 0.140 | 34.0 | cold scan; fresh=0; cached=0 |
| views | combo-kinds | ok | 0 | 0.060 | 24.7 | cold scan; fresh=0; cached=0 |
| views | exclude-ignored-summary | ok | 0 | 0.040 | 15.9 | cold scan; fresh=0; cached=0 |
| views | depth-limit-tree | ok | 0 | 0.030 | 16.7 | cold scan; fresh=0; cached=0 |
| views | scan-depth-1-summary | ok | 0 | 0.020 | 14.4 | cold scan; fresh=0; cached=0 |
| views | json-summary | ok | 0 | 0.040 | 15.3 |  |
| views | yaml-summary | ok | 0 | 0.040 | 15.5 |  |
| cache-analyze | off-code-1 | ok | 0 | 0.980 | 35.7 | cold scan; fresh=13754; cached=0 |
| cache-analyze | off-code-2 | ok | 0 | 1.000 | 35.7 | cold scan; fresh=13754; cached=0 |
| cache-analyze | off-lines-1 | ok | 0 | 0.510 | 35.6 | cold scan; fresh=13754; cached=0 |
| cache-analyze | off-lines-2 | ok | 0 | 0.520 | 34.4 | cold scan; fresh=13754; cached=0 |
| cache-analyze | off-cache-status | ok | 0 | 0.020 | 13.5 |  |
| cache-analyze | on-code-1 | ok | 0 | 0.960 | 37.9 | cold scan; fresh=13754; cached=0 |
| cache-analyze | on-code-2 | ok | 0 | 0.160 | 39.6 | warm revalidation; fresh=0; cached=13754 |
| cache-analyze | on-lines-1 | ok | 0 | 0.540 | 42.5 | warm revalidation; fresh=13754; cached=0 |
| cache-analyze | on-lines-2 | ok | 0 | 0.160 | 39.1 | warm revalidation; fresh=0; cached=13754 |
| cache-analyze | on-cache-status | ok | 0 | 0.020 | 13.5 |  |
| analyze-extra | analyze-words | ok | 0 | 0.610 | 45.4 | warm revalidation; fresh=13754; cached=0 |
| analyze-extra | analyze-all | ok | 0 | 1.070 | 49.0 | warm revalidation; fresh=13754; cached=0 |
| analyze-extra | json-analyze-code | ok | 0 | 1.050 | 40.4 | physical_lines=2705144 |
| analyze-extra | yaml-analyze-lines | ok | 0 | 0.530 | 38.9 |  |
| analyze-extra | text-analyze-code-summary | ok | 0 | 0.980 | 39.7 | warm revalidation; fresh=13754; cached=0 |
| watch | watch-sigint | ok | -2 | 3.005 |  | SIGINT after file create; elapsed=3.00s |
| medium | med-tree-cold | ok | 0 | 0.170 | 18.6 | cold scan; fresh=0; cached=0 |
| medium | med-tree-warm | ok | 0 | 0.090 | 19.9 | cold scan; fresh=0; cached=0 |
| medium | med-summary | ok | 0 | 0.090 | 21.8 | cold scan; fresh=0; cached=0 |
| medium | med-languages | ok | 0 | 0.190 | 58.4 | cold scan; fresh=0; cached=0 |
| medium | med-combo-kinds | ok | 0 | 0.210 | 59.2 | cold scan; fresh=0; cached=0 |
| medium | med-recent | ok | 0 | 0.220 | 66.6 | cold scan; fresh=0; cached=0 |
| medium | med-json-summary | ok | 0 | 0.090 | 22.0 |  |
| medium | med-analyze-code-subdir | ok | 0 | 0.210 | 30.7 | cold scan; fresh=10121; cached=0 |
| medium | med-analyze-code-subdir-warm | ok | 0 | 0.080 | 29.5 | warm revalidation; fresh=0; cached=10121 |
| large | large-summary-depth1 | ok | 0 | 0.020 | 14.4 | cold scan; fresh=0; cached=0 |
| large | large-summary-depth2 | ok | 0 | 0.020 | 14.7 | cold scan; fresh=0; cached=0 |

### Peer agreement self-test

#### `<scratch>/probe root`

fdu read 44.0 KiB allocated and 1.0 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 1 hard-linked inodes, which per-path counting adds 24.0 KiB
allocated to; 4 symbolic links, 1.0 KiB apparent; 5 directories, 20.0 KiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 68.0 KiB | +24.0 KiB | +24.0 KiB | symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| GNU du | allocated | 44.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| GNU du -l | apparent | 1.0 GiB | +1.0 KiB | +1.0 KiB | symbolic links +1.0 KiB | agrees exactly | none | 0.0 s |
| GNU du | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | none | 0.0 s |
| dust | allocated | 44.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| dust | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | symbolic links +1.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 68.0 KiB | +24.0 KiB | +24.0 KiB | symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 1.0 GiB | +21.0 KiB | +21.0 KiB | symbolic links +1.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| dua | allocated | 36.0 KiB | -8.0 KiB | -8.0 KiB | hard links once -24.0 KiB, directories +16.0 KiB | agrees exactly | none | 0.0 s |
| dua | apparent | 1.0 GiB | -3.5 KiB | -3.5 KiB | hard links once -19.5 KiB, symbolic links +37 B, directories +16.0 KiB | agrees exactly | none | 0.0 s |
| diskus | allocated | 44.0 KiB | 0 B | 0 B | hard links once -24.0 KiB, symbolic links +4.0 KiB, directories +20.0 KiB | agrees exactly | none | 0.0 s |
| diskus | apparent | 1.0 GiB | -18.5 KiB | -18.5 KiB | hard links once -19.5 KiB, symbolic links +1.0 KiB | agrees exactly | none | 0.0 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 4 of
4\.

| Directory | fdu | GNU du -l | Δ |
| --- | ---: | ---: | ---: |
| `a` | 24.0 KiB to 24.0 KiB | 28.0 KiB | +4.0 KiB |
| `b` | 12.0 KiB to 12.0 KiB | 16.0 KiB | +4.0 KiB |
| `locked` | 4.0 KiB to 4.0 KiB | 8.0 KiB | +4.0 KiB |
| `name with spaces` | 4.0 KiB to 4.0 KiB | 8.0 KiB | +4.0 KiB |

Self-test: 12 readings agree exactly; 4 failed.
Not installed, so not checked: BSD du allocated self-test exit status: 1

### Peer agreement on real trees

#### `<fdu checkout>`

fdu read 422.3 MiB allocated and 387.6 MiB apparent, exiting 0; across its 13 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 21 symbolic links, 236 B apparent; 2,479 directories, 9.7 MiB apparent; directories
that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 432.0 MiB | +9.7 MiB | +9.7 MiB | directories +9.7 MiB | agrees exactly | none | 0.1 s |
| GNU du | allocated | 432.0 MiB | +9.7 MiB | +9.7 MiB | directories +9.7 MiB | agrees exactly | none | 0.1 s |
| GNU du -l | apparent | 387.6 MiB | +236 B | +236 B | symbolic links +236 B | agrees exactly | none | 0.1 s |
| GNU du | apparent | 387.6 MiB | +236 B | +236 B | symbolic links +236 B | agrees exactly | none | 0.1 s |
| dust | allocated | 432.0 MiB | +9.7 MiB | +9.7 MiB | directories +9.7 MiB | agrees exactly | none | 0.0 s |
| dust | apparent | 397.3 MiB | +9.7 MiB | +9.7 MiB | symbolic links +236 B, directories +9.7 MiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 432.0 MiB | +9.7 MiB | +9.7 MiB | directories +9.7 MiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 397.3 MiB | +9.7 MiB | +9.7 MiB | symbolic links +236 B, directories +9.7 MiB | agrees exactly | none | 0.0 s |
| dua | allocated | 432.0 MiB | +9.7 MiB | +9.7 MiB | directories +9.7 MiB | agrees exactly | none | 0.1 s |
| dua | apparent | 397.3 MiB | +9.7 MiB | +9.7 MiB | symbolic links +236 B, directories +9.7 MiB | agrees exactly | none | 0.1 s |
| diskus | allocated | 432.0 MiB | +9.7 MiB | +9.7 MiB | directories +9.7 MiB | agrees exactly | none | 0.0 s |
| diskus | apparent | 387.6 MiB | +236 B | +236 B | symbolic links +236 B | agrees exactly | none | 0.0 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 11 of
11\.

| Directory | fdu | GNU du -l | Δ |
| --- | ---: | ---: | ---: |
| `.agents` | 44.0 KiB to 44.0 KiB | 60.0 KiB | +16.0 KiB |
| `.claude` | 60.0 KiB to 60.0 KiB | 84.0 KiB | +24.0 KiB |
| `.codex` | 12.0 KiB to 12.0 KiB | 16.0 KiB | +4.0 KiB |
| `.github` | 56.0 KiB to 56.0 KiB | 64.0 KiB | +8.0 KiB |
| `.tbd` | 20.0 KiB to 20.0 KiB | 24.0 KiB | +4.0 KiB |
| `crates` | 353.3 MiB to 353.3 MiB | 361.4 MiB | +8.2 MiB |
| `docs` | 22.2 MiB to 22.2 MiB | 22.4 MiB | +260.0 KiB |
| `explorations` | 22.6 MiB to 22.6 MiB | 23.1 MiB | +504.0 KiB |
| `node_modules` | 21.3 MiB to 21.3 MiB | 21.9 MiB | +584.0 KiB |
| `scripts` | 880.0 KiB to 880.0 KiB | 896.0 KiB | +16.0 KiB |
| `tests` | 1.6 MiB to 1.6 MiB | 1.8 MiB | +172.0 KiB |

#### `~/.rustup`

fdu read 2.4 GiB allocated and 2.4 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 0 symbolic links, 0 B apparent; 82 directories, 328.0 KiB apparent; directories that
could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| GNU du | allocated | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| GNU du -l | apparent | 2.4 GiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| GNU du | apparent | 2.4 GiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |
| dust | allocated | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| dust | apparent | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| pdu | allocated | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| pdu | apparent | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| dua | allocated | 2.4 GiB | +324.0 KiB | +324.0 KiB | directories +324.0 KiB | agrees exactly | none | 0.0 s |
| dua | apparent | 2.4 GiB | +324.0 KiB | +324.0 KiB | directories +324.0 KiB | agrees exactly | none | 0.0 s |
| diskus | allocated | 2.4 GiB | +328.0 KiB | +328.0 KiB | directories +328.0 KiB | agrees exactly | none | 0.0 s |
| diskus | apparent | 2.4 GiB | 0 B | 0 B | — | agrees exactly | none | 0.0 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 4 of
4\.

| Directory | fdu | GNU du -l | Δ |
| --- | ---: | ---: | ---: |
| `downloads` | 0 B to 0 B | 4.0 KiB | +4.0 KiB |
| `tmp` | 0 B to 0 B | 4.0 KiB | +4.0 KiB |
| `toolchains` | 2.4 GiB to 2.4 GiB | 2.4 GiB | +312.0 KiB |
| `update-hashes` | 12.0 KiB to 12.0 KiB | 16.0 KiB | +4.0 KiB |

#### `/usr`

fdu read 3.5 GiB allocated and 3.3 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 5 hard-linked inodes, which per-path counting adds 4.1 MiB
allocated to; 8,559 symbolic links, 183.4 KiB apparent; 7,845 directories, 32.0 MiB
apparent; directories that could not be listed: none; entries that could not be stat’d:
0\.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 3.5 GiB | +32.1 MiB | +32.1 MiB | symbolic links +48.0 KiB, directories +32.1 MiB | agrees exactly | none | 0.2 s |
| GNU du | allocated | 3.5 GiB | +28.0 MiB | +28.0 MiB | hard links once -4.1 MiB, symbolic links +48.0 KiB, directories +32.1 MiB | agrees exactly | none | 0.2 s |
| GNU du -l | apparent | 3.3 GiB | +183.4 KiB | +183.4 KiB | symbolic links +183.4 KiB | agrees exactly | none | 0.2 s |
| GNU du | apparent | 3.3 GiB | -3.9 MiB | -3.9 MiB | hard links once -4.1 MiB, symbolic links +183.4 KiB | agrees exactly | none | 0.2 s |
| dust | allocated | 3.5 GiB | +28.0 MiB | +28.0 MiB | hard links once -4.1 MiB, symbolic links +48.0 KiB, directories +32.1 MiB | agrees exactly | none | 0.1 s |
| dust | apparent | 3.4 GiB | +32.2 MiB | +32.2 MiB | symbolic links +183.4 KiB, directories +32.0 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 3.5 GiB | +32.1 MiB | +32.1 MiB | symbolic links +48.0 KiB, directories +32.1 MiB | agrees exactly | none | 0.1 s |
| pdu | apparent | 3.4 GiB | +32.2 MiB | +32.2 MiB | symbolic links +183.4 KiB, directories +32.0 MiB | agrees exactly | none | 0.1 s |
| dua | allocated | 3.5 GiB | +28.0 MiB | +28.0 MiB | hard links once -4.1 MiB, symbolic links +48.0 KiB, directories +32.1 MiB | agrees exactly | none | 0.3 s |
| dua | apparent | 3.4 GiB | +28.0 MiB | +28.0 MiB | hard links once -4.1 MiB, symbolic links +183.4 KiB, directories +32.0 MiB | agrees exactly | none | 0.4 s |
| diskus | allocated | 3.5 GiB | +28.0 MiB | +28.0 MiB | hard links once -4.1 MiB, symbolic links +48.0 KiB, directories +32.1 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 3.3 GiB | -3.9 MiB | -3.9 MiB | hard links once -4.1 MiB, symbolic links +183.4 KiB | agrees exactly | none | 0.1 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 10 of
10\.

| Directory | fdu | GNU du -l | Δ |
| --- | ---: | ---: | ---: |
| `bin` | 418.5 MiB to 418.5 MiB | 418.5 MiB | +40.0 KiB |
| `games` | 0 B to 0 B | 4.0 KiB | +4.0 KiB |
| `include` | 51.4 MiB to 51.4 MiB | 52.3 MiB | +892.0 KiB |
| `lib` | 1.9 GiB to 1.9 GiB | 1.9 GiB | +6.8 MiB |
| `lib64` | 0 B to 0 B | 4.0 KiB | +4.0 KiB |
| `libexec` | 264.8 MiB to 264.8 MiB | 264.9 MiB | +48.0 KiB |
| `local` | 554.5 MiB to 554.5 MiB | 567.5 MiB | +13.0 MiB |
| `sbin` | 6.6 MiB to 6.6 MiB | 6.6 MiB | +4.0 KiB |
| `share` | 313.6 MiB to 313.6 MiB | 324.9 MiB | +11.3 MiB |
| `src` | 72.0 KiB to 72.0 KiB | 88.0 KiB | +16.0 KiB |

#### `linux-v6.12`

fdu read 1.8 GiB allocated and 1.6 GiB apparent, exiting 0; across its 13 readings, one
before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 62 symbolic links, 2.0 KiB apparent; 5,769 directories, 23.2 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.3 s |
| GNU du | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | 1.6 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.3 s |
| GNU du | apparent | 1.6 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.3 s |
| dust | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.2 s |
| dust | apparent | 1.7 GiB | +23.2 MiB | +23.2 MiB | symbolic links +2.0 KiB, directories +23.2 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.1 s |
| pdu | apparent | 1.7 GiB | +23.2 MiB | +23.2 MiB | symbolic links +2.0 KiB, directories +23.2 MiB | agrees exactly | none | 0.1 s |
| dua | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.3 s |
| dua | apparent | 1.7 GiB | +23.2 MiB | +23.2 MiB | symbolic links +2.0 KiB, directories +23.2 MiB | agrees exactly | none | 0.3 s |
| diskus | allocated | 1.9 GiB | +23.2 MiB | +23.2 MiB | directories +23.2 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 1.6 GiB | +2.0 KiB | +2.0 KiB | symbolic links +2.0 KiB | agrees exactly | none | 0.1 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 25 of
25\.

| Directory | fdu | GNU du -l | Δ |
| --- | ---: | ---: | ---: |
| `.git` | 269.8 MiB to 269.8 MiB | 269.8 MiB | +44.0 KiB |
| `Documentation` | 70.5 MiB to 70.5 MiB | 73.3 MiB | +2.8 MiB |
| `LICENSES` | 272.0 KiB to 272.0 KiB | 292.0 KiB | +20.0 KiB |
| `arch` | 148.9 MiB to 148.9 MiB | 152.7 MiB | +3.8 MiB |
| `block` | 2.1 MiB to 2.1 MiB | 2.1 MiB | +8.0 KiB |
| `certs` | 76.0 KiB to 76.0 KiB | 80.0 KiB | +4.0 KiB |
| `crypto` | 3.8 MiB to 3.8 MiB | 3.8 MiB | +12.0 KiB |
| `drivers` | 1.0 GiB to 1.0 GiB | 1.0 GiB | +9.3 MiB |
| `fs` | 50.2 MiB to 50.2 MiB | 50.6 MiB | +400.0 KiB |
| `include` | 54.5 MiB to 54.5 MiB | 55.9 MiB | +1.3 MiB |
| `init` | 196.0 KiB to 196.0 KiB | 200.0 KiB | +4.0 KiB |
| `io_uring` | 704.0 KiB to 704.0 KiB | 708.0 KiB | +4.0 KiB |
| `ipc` | 276.0 KiB to 276.0 KiB | 280.0 KiB | +4.0 KiB |
| `kernel` | 14.1 MiB to 14.1 MiB | 14.2 MiB | +112.0 KiB |
| `lib` | 8.3 MiB to 8.3 MiB | 8.4 MiB | +108.0 KiB |
| `mm` | 5.7 MiB to 5.7 MiB | 5.7 MiB | +24.0 KiB |
| `net` | 36.6 MiB to 36.6 MiB | 37.0 MiB | +384.0 KiB |
| `rust` | 828.0 KiB to 828.0 KiB | 892.0 KiB | +64.0 KiB |
| `samples` | 1.6 MiB to 1.6 MiB | 1.8 MiB | +172.0 KiB |
| `scripts` | 4.2 MiB to 4.2 MiB | 4.4 MiB | +216.0 KiB |
| `security` | 3.6 MiB to 3.6 MiB | 3.7 MiB | +92.0 KiB |
| `sound` | 50.4 MiB to 50.4 MiB | 51.1 MiB | +712.0 KiB |
| `tools` | 81.0 MiB to 81.0 MiB | 84.6 MiB | +3.6 MiB |
| `usr` | 68.0 KiB to 68.0 KiB | 80.0 KiB | +12.0 KiB |
| `virt` | 316.0 KiB to 316.0 KiB | 328.0 KiB | +12.0 KiB |

#### `node-modules-dense`

fdu read 722.8 MiB allocated and 514.0 MiB apparent, exiting 0; across its 13 readings,
one before each tool and one at the end, the tree moved 0 B allocated and 0 B apparent.
fdu errors: none.

Measured from the tree: 0 hard-linked inodes, which per-path counting adds 0 B allocated
to; 107 symbolic links, 2.6 KiB apparent; 9,439 directories, 37.5 MiB apparent;
directories that could not be listed: none; entries that could not be stat’d: 0.

| Tool | Metric | Total | Δ vs fdu | Expected Δ | Made of | Verdict | Errors | Time |
| --- | --- | ---: | ---: | ---: | --- | --- | --- | ---: |
| GNU du -l | allocated | 760.3 MiB | +37.5 MiB | +37.5 MiB | directories +37.5 MiB | agrees exactly | none | 0.3 s |
| GNU du | allocated | 760.3 MiB | +37.5 MiB | +37.5 MiB | directories +37.5 MiB | agrees exactly | none | 0.3 s |
| GNU du -l | apparent | 514.0 MiB | +2.6 KiB | +2.6 KiB | symbolic links +2.6 KiB | agrees exactly | none | 0.3 s |
| GNU du | apparent | 514.0 MiB | +2.6 KiB | +2.6 KiB | symbolic links +2.6 KiB | agrees exactly | none | 0.3 s |
| dust | allocated | 760.3 MiB | +37.5 MiB | +37.5 MiB | directories +37.5 MiB | agrees exactly | none | 0.2 s |
| dust | apparent | 551.6 MiB | +37.5 MiB | +37.5 MiB | symbolic links +2.6 KiB, directories +37.5 MiB | agrees exactly | none | 0.1 s |
| pdu | allocated | 760.3 MiB | +37.5 MiB | +37.5 MiB | directories +37.5 MiB | agrees exactly | none | 0.1 s |
| pdu | apparent | 551.6 MiB | +37.5 MiB | +37.5 MiB | symbolic links +2.6 KiB, directories +37.5 MiB | agrees exactly | none | 0.1 s |
| dua | allocated | 760.3 MiB | +37.5 MiB | +37.5 MiB | directories +37.5 MiB | agrees exactly | none | 0.3 s |
| dua | apparent | 551.6 MiB | +37.5 MiB | +37.5 MiB | symbolic links +2.6 KiB, directories +37.5 MiB | agrees exactly | none | 0.3 s |
| diskus | allocated | 760.3 MiB | +37.5 MiB | +37.5 MiB | directories +37.5 MiB | agrees exactly | none | 0.1 s |
| diskus | apparent | 514.0 MiB | +2.6 KiB | +2.6 KiB | symbolic links +2.6 KiB | agrees exactly | none | 0.1 s |

Top-level directories whose allocated size does not agree with GNU du -l exactly: 1 of
1\.

| Directory | fdu | GNU du -l | Δ |
| --- | ---: | ---: | ---: |
| `node_modules` | 721.8 MiB to 721.8 MiB | 759.4 MiB | +37.5 MiB |

51 readings failed. peer agreement exit status: 1

Separate walk of every top-level row (fdu-83km):

```
<fdu checkout>: 11 of 11 top-level rows equal fdu + the subtree's non-regular blocks (fdu unchanged across the bracket: True)
~/.rustup: 4 of 4 top-level rows equal fdu + the subtree's non-regular blocks (fdu unchanged across the bracket: True)
/usr: 10 of 10 top-level rows equal fdu + the subtree's non-regular blocks (fdu unchanged across the bracket: True)
linux-v6.12: 25 of 25 top-level rows equal fdu + the subtree's non-regular blocks (fdu unchanged across the bracket: True)
node-modules-dense: 1 of 1 top-level rows equal fdu + the subtree's non-regular blocks (fdu unchanged across the bracket: True)
total rows checked: 51; mismatches or one-sided: 0
```

### Phase 6 pty probe

```

ok   slow scan exits 0 (1.5 s)
ok   slow scan draws frames (11 frames)
ok   first frame after about half a second (0.52 s)
ok   a frame says Scanning
     phases seen: ['Scanning', 'Summarizing']
ok   the last frame is erased before the report
ok   the report follows the erase, with no frame left in it
ok   every frame fits 100 columns (at most 99)
ok   no frame contains a newline
ok   file counts climb ([376004] to [1172718])
ok   the counts hold their column while Scanning (positions [38])
     sample frame: ⠴ <home>  Scanning        781,782 files ·   117,962 dirs ·   17 GiB · 1.0 s
ok   a small tree shows no indicator
ok   redirected stderr holds no carriage return or escape
ok   redirected stderr holds only note/warn/tip/perf lines (6 lines)
ok   nothing drawn on the terminal when stderr is a file
ok   --format json shows nothing
ok   --progress always --format json draws and erases
ok   --progress never shows nothing (text)
ok   --progress never shows nothing (json)
ok   --progress never shows nothing (yaml)
ok   CI=1 shows nothing
ok   TERM=dumb shows nothing
ok   NO_COLOR=1 keeps the animation
ok   NO_COLOR=1 frames carry no color
ok   without NO_COLOR the frames are colored
ok   Ctrl-C: dies by SIGINT (status 2; shell $? would be 130)
ok   Ctrl-C: prints fdu: interrupted
ok   Ctrl-C: the line is erased before the message
ok   Ctrl-C: no frame after the message
ok   60 columns: 59 frames fit, widest 59 ('⠇ <home>/…ivers  Summarizing  34,957 files · 1.0 GiB · 6.3 s')
ok   40 columns: 58 frames fit, widest 13 ('⠧ Summarizing')
ok   30 columns: 57 frames fit, widest 13 ('⠦ Summarizing')
ok   20 columns: 57 frames fit, widest 13 ('⠦ Summarizing')
ok   19 columns: 58 frames fit, spinner and phase word only ('⠧ Summarizing')
ok   12 columns: 56 frames fit, spinner and phase word only ('⠋ Analyzing')
ok   narrowing mid-run shrinks every later frame without wrapping
ok   --analyze all shows Analyzing with a percentage ([13] to [98])
ok   the percentage climbs
ok   the analyze indicator is erased before the report

38 of 38 checks passed
```

### Correctness: corr-refusals

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     2      2      1  cold_scan        -            -       withheld
summary                     2      2      1  cold_scan        -            -       withheld
tree                        2      2      1  cold_scan        -            -       withheld
types                       2      2      1  cold_scan        -            -       withheld
extensions                  2      2      1  cold_scan        -            -       withheld
largest                     2      2      1  cold_scan        -            -       withheld
files                       2      2      1  cold_scan        -            -       withheld
size-apparent               2      2      1  cold_scan        -            -       withheld
size-allocated              2      2      1  cold_scan        -            -       withheld
sort-mtime                  2      2      1  cold_scan        -            -       withheld
sort-name                   2      2      1  cold_scan        -            -       withheld
depth-1                     2      2      1  cold_scan        -            -       withheld
scan-depth-2                2      2      1  cold_scan        -            -       withheld
min-size                    2      2      1  cold_scan        -            -       withheld
no-gitignore                2      2      1  cold_scan        -            -       withheld
exclude-ignored             2      2      1  cold_scan        -            -       withheld
one-filesystem              2      2      1  cold_scan        -            -       withheld
analyze-lines               2      2      1  cold_scan        -            -       withheld
analyze-code                2      2      1  cold_scan        -            -       withheld
analyze-words               2      2      1  cold_scan        -            -       withheld
analyze-all                 2      2      1  cold_scan        -            -       withheld
languages                   2      2      1  cold_scan        -            -       withheld
documents                   2      2      1  cold_scan        -            -       withheld

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 0 of 23
warm_cold.py --refusals-only exit status: 0
```

### Correctness: corr-served

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     0      0      0  cold_scan        cache_only   stale   ok
summary                     0      0      0  cold_scan        cache_only   stale   ok
tree                        0      0      0  cold_scan        cache_only   stale   ok
types                       0      0      0  cold_scan        cache_only   stale   ok
extensions                  0      0      0  cold_scan        cache_only   stale   ok
largest                     0      0      0  cold_scan        cache_only   stale   ok
files                       0      0      0  cold_scan        cache_only   stale   ok
size-apparent               0      0      0  cold_scan        cache_only   stale   ok
size-allocated              0      0      0  cold_scan        cache_only   stale   ok
sort-mtime                  0      0      0  cold_scan        cache_only   stale   ok
sort-name                   0      0      0  cold_scan        cache_only   stale   ok
depth-1                     0      0      0  cold_scan        cache_only   stale   ok
scan-depth-2                0      0      0  cold_scan        cache_only   stale   ok
min-size                    0      0      0  cold_scan        cache_only   stale   ok
no-gitignore                0      0      0  cold_scan        cache_only   stale   ok
exclude-ignored             0      0      0  cold_scan        cache_only   stale   ok
one-filesystem              0      0      0  cold_scan        cache_only   stale   ok
analyze-lines               0      0      0  warm_revalidate  cache_only   stale   ok
analyze-code                0      0      0  warm_revalidate  cache_only   stale   ok
analyze-words               0      0      0  warm_revalidate  cache_only   stale   ok
analyze-all                 0      0      0  warm_revalidate  cache_only   stale   ok
languages                   0      0      0  warm_revalidate  cache_only   stale   ok
documents                   0      0      0  warm_revalidate  cache_only   stale   ok

answer mismatches: 0
mechanism failures (cache did not serve): 0
stale reference instants: 0
cases the snapshot served: 23 of 23
warm_cold.py exit status: 0
```

### Correctness: corr-cross

```
warmer   ask                   rc  analysis.analyze           verdict
----------------------------------------------------------------------------------
W_none   a_lines                0  ['lines']                  ok
W_none   a_code                 0  ['lines', 'code']          ok
W_none   a_words                0  ['lines', 'words']         ok
W_none   a_all                  0  ['lines', 'code', 'words'] ok
W_none   a_lines_documents      0  ['lines']                  ok
W_none   a_code_languages       0  ['lines', 'code']          ok
W_lines  a_lines                0  ['lines']                  ok
W_lines  a_code                 0  ['lines', 'code']          ok
W_lines  a_words                0  ['lines', 'words']         ok
W_lines  a_all                  0  ['lines', 'code', 'words'] ok
W_lines  a_lines_documents      0  ['lines']                  ok
W_lines  a_code_languages       0  ['lines', 'code']          ok
W_code   a_lines                0  ['lines']                  ok
W_code   a_code                 0  ['lines', 'code']          ok
W_code   a_words                0  ['lines', 'words']         ok
W_code   a_all                  0  ['lines', 'code', 'words'] ok
W_code   a_lines_documents      0  ['lines']                  ok
W_code   a_code_languages       0  ['lines', 'code']          ok
W_words  a_lines                0  ['lines']                  ok
W_words  a_code                 0  ['lines', 'code']          ok
W_words  a_words                0  ['lines', 'words']         ok
W_words  a_all                  0  ['lines', 'code', 'words'] ok
W_words  a_lines_documents      0  ['lines']                  ok
W_words  a_code_languages       0  ['lines', 'code']          ok
W_all    a_lines                0  ['lines']                  ok
W_all    a_code                 0  ['lines', 'code']          ok
W_all    a_words                0  ['lines', 'words']         ok
W_all    a_all                  0  ['lines', 'code', 'words'] ok
W_all    a_lines_documents      0  ['lines']                  ok
W_all    a_code_languages       0  ['lines', 'code']          ok

cross-warm violations: 0
cross_warm.py exit status: 0
```

### Correctness: corr-break1-warm-cold

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
summary                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
tree                        0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
types                       0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
extensions                  0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
largest                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
files                       0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
size-apparent               0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
size-allocated              0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
sort-mtime                  0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
sort-name                   0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
depth-1                     0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
scan-depth-2                0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
min-size                    0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
no-gitignore                0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
exclude-ignored             0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
one-filesystem              0      0      1  cold_scan        -            -       NO-SNAPSHOT(rc=1)
analyze-lines               0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
analyze-code                0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
analyze-words               0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
analyze-all                 0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
languages                   0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)
documents                   0      0      0  cold_scan        cache_only   stale   NOT-WARM(scanned)

answer mismatches: 0
mechanism failures (cache did not serve): 23
  - default: cache-only exited 1, so nothing was stored
  - summary: cache-only exited 1, so nothing was stored
  - tree: cache-only exited 1, so nothing was stored
  - types: cache-only exited 1, so nothing was stored
  - extensions: cache-only exited 1, so nothing was stored
  - largest: cache-only exited 1, so nothing was stored
  - files: cache-only exited 1, so nothing was stored
  - size-apparent: cache-only exited 1, so nothing was stored
  - size-allocated: cache-only exited 1, so nothing was stored
  - sort-mtime: cache-only exited 1, so nothing was stored
  - sort-name: cache-only exited 1, so nothing was stored
  - depth-1: cache-only exited 1, so nothing was stored
  - scan-depth-2: cache-only exited 1, so nothing was stored
  - min-size: cache-only exited 1, so nothing was stored
  - no-gitignore: cache-only exited 1, so nothing was stored
  - exclude-ignored: cache-only exited 1, so nothing was stored
  - one-filesystem: cache-only exited 1, so nothing was stored
  - analyze-lines: warm content tier scanned
  - analyze-code: warm content tier scanned
  - analyze-words: warm content tier scanned
  - analyze-all: warm content tier scanned
  - languages: warm content tier scanned
  - documents: warm content tier scanned
stale reference instants: 0
cases the snapshot served: 6 of 23
warm_cold.py (break: no snapshot) exit status: 1
```

### Correctness: corr-break1-cross

```
warmer   ask                   rc  analysis.analyze           verdict
----------------------------------------------------------------------------------
W_none   a_lines                0  ['lines']                  ok
W_none   a_code                 0  ['lines', 'code']          ok
W_none   a_words                0  ['lines', 'words']         ok
W_none   a_all                  0  ['lines', 'code', 'words'] ok
W_none   a_lines_documents      0  ['lines']                  ok
W_none   a_code_languages       0  ['lines', 'code']          ok
W_lines  a_lines                0  ['lines']                  NOT-WARM(scanned)
W_lines  a_code                 0  ['lines', 'code']          ok
W_lines  a_words                0  ['lines', 'words']         ok
W_lines  a_all                  0  ['lines', 'code', 'words'] ok
W_lines  a_lines_documents      0  ['lines']                  NOT-WARM(scanned)
W_lines  a_code_languages       0  ['lines', 'code']          ok
W_code   a_lines                0  ['lines']                  ok
W_code   a_code                 0  ['lines', 'code']          NOT-WARM(scanned)
W_code   a_words                0  ['lines', 'words']         ok
W_code   a_all                  0  ['lines', 'code', 'words'] ok
W_code   a_lines_documents      0  ['lines']                  ok
W_code   a_code_languages       0  ['lines', 'code']          NOT-WARM(scanned)
W_words  a_lines                0  ['lines']                  ok
W_words  a_code                 0  ['lines', 'code']          ok
W_words  a_words                0  ['lines', 'words']         NOT-WARM(scanned)
W_words  a_all                  0  ['lines', 'code', 'words'] ok
W_words  a_lines_documents      0  ['lines']                  ok
W_words  a_code_languages       0  ['lines', 'code']          ok
W_all    a_lines                0  ['lines']                  ok
W_all    a_code                 0  ['lines', 'code']          ok
W_all    a_words                0  ['lines', 'words']         ok
W_all    a_all                  0  ['lines', 'code', 'words'] NOT-WARM(scanned)
W_all    a_lines_documents      0  ['lines']                  ok
W_all    a_code_languages       0  ['lines', 'code']          ok

cross-warm violations: 6
  - W_lines -> a_lines: NOT-WARM(scanned)
  - W_lines -> a_lines_documents: NOT-WARM(scanned)
  - W_code -> a_code: NOT-WARM(scanned)
  - W_code -> a_code_languages: NOT-WARM(scanned)
  - W_words -> a_words: NOT-WARM(scanned)
  - W_all -> a_all: NOT-WARM(scanned)
cross_warm.py (break: no snapshot) exit status: 1
```

### Correctness: corr-break2-refusals

```
case                     cold   warm   only  warm source      only source  fresh   verdict
--------------------------------------------------------------------------------------------------------
default                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
summary                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
tree                        2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
types                       2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
extensions                  2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
largest                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
files                       2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
size-apparent               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
size-allocated              2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
sort-mtime                  2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
sort-name                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
depth-1                     2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
scan-depth-2                2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
min-size                    2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
no-gitignore                2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
exclude-ignored             2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
one-filesystem              2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-lines               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-code                2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-words               2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
analyze-all                 2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
languages                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED
documents                   2      2      2  cold_scan        cache_only   stale   PARTIAL-STORED

answer mismatches: 0
mechanism failures (cache did not serve): 23
  - default: a partial scan was served from the cache
  - summary: a partial scan was served from the cache
  - tree: a partial scan was served from the cache
  - types: a partial scan was served from the cache
  - extensions: a partial scan was served from the cache
  - largest: a partial scan was served from the cache
  - files: a partial scan was served from the cache
  - size-apparent: a partial scan was served from the cache
  - size-allocated: a partial scan was served from the cache
  - sort-mtime: a partial scan was served from the cache
  - sort-name: a partial scan was served from the cache
  - depth-1: a partial scan was served from the cache
  - scan-depth-2: a partial scan was served from the cache
  - min-size: a partial scan was served from the cache
  - no-gitignore: a partial scan was served from the cache
  - exclude-ignored: a partial scan was served from the cache
  - one-filesystem: a partial scan was served from the cache
  - analyze-lines: a partial scan was served from the cache
  - analyze-code: a partial scan was served from the cache
  - analyze-words: a partial scan was served from the cache
  - analyze-all: a partial scan was served from the cache
  - languages: a partial scan was served from the cache
  - documents: a partial scan was served from the cache
stale reference instants: 0
cases the snapshot served: 0 of 23
warm_cold.py --refusals-only (break: partial stored) exit status: 1
```

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
