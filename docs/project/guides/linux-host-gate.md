# Linux Host Gate: What a Real Linux Machine Proves Before Merge

CI and a Claude Code web session each prove less than they appear to.
CI runs unprivileged on a fresh runner with no mounts, so it runs the permission
fixtures and the native watch tests, but never a test that needs root, an automount, or
a real home directory.
A web session or a container runs as root on an overlay filesystem, so it reads every
mode-000 file, has to waive the permission fixtures
(`FDU_TEST_ALLOW_NO_PERMISSION_BITS`), and cannot run the correctness runbook’s refusal
pass at all.

This gate is the part neither can run.
It needs one real Linux machine, bare metal or a full virtual machine with its own
kernel, an ordinary user account, and `sudo`. It takes about an hour, most of it
unattended.

## When It Runs

- Before merging a pull request that changes what a walk records or which system calls a
  report makes: permission handling, watch and opened-root observation, device and mount
  handling, `volumes` and capacity, serving routes, or snapshot contents.
- Before tagging a release, as part of the
  [stability pass](release-process.md#stability-pass), which runs its later stages on
  the release commit.

A pull request that needs it says so in its test plan, and is not merged until a run at
its final head is recorded on it.

## What It Proves That Nothing Else Does

| Check | CI | Root sandbox | This gate |
| --- | --- | --- | --- |
| Permission fixtures refuse for real (`EACCES`) | yes | waived | yes, unprivileged |
| Native watch delivers to a fresh watch | yes | often waived | yes |
| Automount triggers: direct, indirect map, kernel automount | skipped (not root) | root, overlay only | yes, root, real kernel |
| Correctness runbook refusal pass and both deliberate breaks | not run | refusal pass impossible | yes |
| Device nodes in the correctness trees | not run | yes | in the release pass, run as root |
| Real filesystems (ext4, btrfs, xfs, tmpfs, NFS) and a real home tree | no | overlay | yes |
| Peer agreement with GNU `du` and other tools on real trees | no | partial | yes |

## Setup

- A checkout of the pull request’s head, with its own Cargo target directory, as
  [AGENTS.md](../../../AGENTS.md#build-and-test) requires, owned by your ordinary user.
- The [toolchain versions](../../../AGENTS.md#toolchain-versions): uv at the Makefile’s
  floor, Rust at `MSRV`, and the rustup targets `make cross-lint` checks.
- GNU `time` at `/usr/bin/time`, and for peer agreement GNU `du`, `dust`, `pdu`, `dua`,
  and `diskus` on `PATH`; the pass records a missing peer as skipped.
- The QA playbook’s trees, as
  [Local Fixtures](../../../tests/qa/cli-installed-e2e.qa.md#local-fixtures) defines
  them: `FDU_QA_SMALL` (this checkout), `FDU_QA_MEDIUM` (a larger working tree),
  `FDU_QA_LARGE` (a hostile wide tree, such as a home directory), and
  `FDU_QA_PEER_TREES`.
- A short `TMPDIR`, such as `/var/tmp/fdu`, because the correctness trees hold a Unix
  socket and a socket path is limited to about 104 bytes.

```shell
export HEAD=<the pull request's head commit, full SHA>
export GATE=/var/tmp/fdu-gate/$HEAD    # outside any checkout
export TMPDIR=/var/tmp/fdu && mkdir -p "$TMPDIR" "$GATE"
git -C <checkout> fetch origin && git -C <checkout> checkout --detach "$HEAD"
```

## Running It

### 1. The stability pass on the pull request’s head, unprivileged

The [stability pass](release-process.md#stability-pass) runs on any commit, in a clean
worktree with its own Cargo target directory.
Run it as your ordinary user, with neither waiver set, so its `make check` runs every
permission fixture and native watch test, and the correctness runbook’s refusal pass and
its break are refused by mode bits for real.
A pull request that prepares a release, setting the new version, runs the whole pass,
with no `--only`. Any other pull request selects every step but the two that belong to
the release commit: `semver-check`, which before the version bump asks whether the tree
could still ship as a patch, and `release-rehearse`.

```shell
env -u FDU_TEST_ALLOW_NO_PERMISSION_BITS -u FDU_TEST_ALLOW_NO_NATIVE_WATCH \
  python3 scripts/release/stability_pass.py --commit "$HEAD" --work-dir "$GATE" \
  --only check,cross-lint,candidate,qa,correctness
```

It runs `make check` and `make cross-lint`, builds and installs the candidate, runs the
QA playbook’s harness, peer agreement, the pty probe, and the terminal tests, then the
correctness runbook’s three passes and both deliberate breaks.
The whole pass must exit 0. With `--only`, the pass still reports on the whole release
pass, so it exits 3: read its report, which must show every selected step passed and
only `semver-check` and `release-rehearse` as not run.
Run unprivileged, the correctness trees have no device nodes, and the report says so; a
root run, as the release pass makes, adds them.

### 2. The full path-independence tier

In the checkout, at the same head:

```shell
make path-independence-full
```

### 3. Automount and mount tests, as root

Build the root-only test as your user, then run the binary as root, so the target
directory never holds a root-owned file:

```shell
cargo test --locked --all-features -p fdu-core --test volumes_automount --no-run 2>&1 | tail -1
sudo <the test binary the line above names> --test-threads=1 --nocapture 2>&1 | tee volumes-automount.log
grep -c '^skipped:' volumes-automount.log    # must print 0
```

Every test must pass, and none may print a `skipped:` line, which `--nocapture` makes
visible: a test that cannot mount returns early and would otherwise read as a pass.
They mount real autofs triggers (direct, an expired one, an indirect map with default
keys, a report rooted at an indirect map), a kernel automount (tracefs under debugfs),
and tmpfs, and count the mount requests each report makes.

### 4. The pull request’s own manual checks

A pull request lists the checks only a real machine answers in its test plan’s **Manual
Testing, Remaining** section; run each one and record what it printed.
For a change to time selection, `ages`, or `volumes`, those are at least:

- `fdu ~ --kind file --since 1d --view summary` and the same with `--time modified`: the
  two clocks’ share of a real home tree, and the same after installing one package into
  two uv or pnpm environments that link from a shared store, which moves the change time
  of every linked name.
- `fdu ~ --view ages` under each clock, and its time and peak memory beside the default
  tree (`/usr/bin/time -v`).
- `fdu / --view volumes` without `--one-filesystem` on a machine with real mounts: each
  row’s type is named, every row’s `counted` adds up to the summary, and an NFS or
  autofs mount, where the machine has one, is asked only while it is mounted.

## Recording a Result

Post one comment on the pull request, at the head it ran, that names the host (kernel,
distribution, bare metal or virtual), the filesystem, each step’s exit status and
counts, the stability pass’s report, and every manual check’s output.
A failure is a finding on the pull request, fixed and rerun at the new head like any
other. Before a release, the stability pass’s report is the record, as the release
process describes.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
