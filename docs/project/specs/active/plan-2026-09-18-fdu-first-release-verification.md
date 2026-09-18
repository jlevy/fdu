# Feature: First-Release Verification

**Date:** 2026-09-18

**Author:** fdu project

**Status:** Active. The pre-registry simulation is recorded on `fdu-bnp9`. Post-publish
items wait until `0.1.0` is on the channels.

## Overview

Two checklists for the first public `0.1.0`. The first can be run before either registry
exists. The second can run only after
[Publishing 0.1.0 by Hand](../../guides/release-process.md#publishing-010-by-hand)
finishes.

The subject is the stranger path: install and run the command, the Rust library, and the
Python package the way a first-time user would, not the way a checkout of this
repository would.

## Goals

- Exercise the packaged CLI, host wheel, and path-crate Rust consumer from local
  artifacts before crates.io or PyPI exist.
- Exercise the same surfaces from crates.io, PyPI, and the GitHub release after `0.1.0`
  is on those channels, from a machine that did not just publish.
- Keep the two phases distinct so a pre-registry step cannot be mistaken for a
  post-publish one.

## Non-Goals

- Publishing `0.1.0`. That procedure is
  [Publishing 0.1.0 by Hand](../../guides/release-process.md#publishing-010-by-hand),
  tracked on `fdu-9cf0`.
- Replacing `make release-rehearse`. Rehearsal already packages and inspects artifacts;
  this plan is the stranger path on top of them.
- Contacting crates.io or PyPI during the first checklist.
- Fixing gaps the simulation already recorded on `origin/main` `98379c76` (`fdu-18vk`,
  `fdu-i142`). Those remain on their own beads.
  Pull request #87 is the leftovers that restore wheel SIGINT, accept whole-millisecond
  `--interval`, and ship a crates.io README with absolute links.

## Background

`make release-rehearse` packages both crates, the host `abi3` wheel, and the source
distribution, then inspects them.
It does not install those artifacts the way a stranger would, and it cannot see registry
pages, docs.rs, or GitHub release assets until they exist.

A 2026-09-18 run against `origin/main` (`98379c76`) is recorded on `fdu-bnp9`. Watch,
Python, Rust, and CLI succeeded from packaged artifacts.
On that revision a wheel-installed `--watch` ignores Ctrl-C (`fdu-18vk`) and relative
README links that crates.io would resolve under `crates/fdu/` 404 (`fdu-i142`). Pull
request #87 is the change that restores SIGINT, accepts whole milliseconds, and ships
`crates/fdu/README.md` with absolute links.

## Design

### Approach

Run the first checklist from packaged local artifacts only.
Run the second from a clean machine or container that has no `CARGO_REGISTRY_TOKEN`, no
`UV_PUBLISH_TOKEN`, and no checkout of this repository.
Use a GIL-enabled CPython 3.12 (and again 3.14) and a Rust toolchain ≥ 1.85.

### Components

- Command line: `cargo install` / `uv tool` / `uvx` / `pip`, then `--help`, `--docs`,
  default tree, summary, and `--watch`
- Rust library: `cargo add fdu` (and `fdu-core --features watch`) against published
  crates, or path crates before they exist
- Python package: host wheel before publish; sdist, five wheels, `import fdu`, and
  `index.watch()` after
- GitHub release: tag, notes body, and the 11 named assets

### API Changes

None. This plan verifies existing surfaces.

## Implementation Plan

### Phase 1: Before the Registries Exist

These steps use only packaged local artifacts: `cargo package` for both crates, the host
`abi3` wheel, and the source distribution.
They do not contact crates.io or PyPI. `make release-rehearse` already packages and
inspects those artifacts; [the new-user simulation](#new-user-simulation) is the
stranger path on top of them.

#### New-User Simulation

Work in a scratch directory that is not the checkout.

1. **Package both crates together**, then install the packaged `fdu` against the
   packaged `fdu-core` with `scripts/release/smoke_crate.py`. Until `fdu-core` is on
   crates.io, a plain `cargo install --locked fdu` cannot resolve it; the script is the
   closest equivalent of `cargo install --locked fdu --version 0.1.0`.

2. **Run the command a stranger runs first:**

   ```shell
   fdu --version          # prints `fdu 0.1.0`
   fdu --help             # bare `fdu` is the same, and scans nothing
   fdu --docs             # offline usage guide
   fdu --cache off .      # default tree
   fdu --view summary --cache off .
   ```

3. **Watch.** Start
   `fdu --watch --view files --format jsonl --cache off --interval 1s TREE`, create a
   file in `TREE`, and confirm a `fdu.stream/1` upsert arrives.
   This simulation used `--interval 1s`. On `98379c76`, `200ms` and `0.2s` are usage
   errors. Pull request #87 accepts whole milliseconds (`200ms`; `0.2s` stays rejected,
   `fdu-8o7g`). Interrupt with Ctrl-C. On that revision a wheel-installed command
   ignores Ctrl-C during `--watch` (`fdu-18vk`); #87 restores SIGINT.

4. **Python, from the host wheel, not a rebuild:**

   ```shell
   uv tool install --no-index --from ./fdu-0.1.0-*.whl --python 3.12 fdu
   uv run --python 3.12 --with ./fdu-0.1.0-*.whl python -c 'import fdu; print(fdu.open(".", cache=fdu.CachePolicy.OFF).total().files)'
   ```

   `uv tool install` puts the console script in an isolated tool env; `uv run` does not
   use that env. Import from the wheel with `--with`. Confirm
   `uv tool run --from that.whl fdu --version` is the same binary identity as the crate
   install.

5. **Rust consumer** against the path crates (`cargo add` cannot resolve `fdu` until
   crates.io has it): `open` a tree, print `index.total().files`.

6. **Registry-page rehearsal.** `crates/fdu/Cargo.toml` `readme` and `crates/fdu-py`
   `[project.urls]` decide what a stranger sees on crates.io and PyPI. Relative links in
   a README that crates.io resolves under `crates/fdu/` will 404. That defect is
   `fdu-i142` / pull request #87, which adds `crates/fdu/README.md` with absolute links.

### Phase 2: After 0.1.0 Is on the Channels

Run these from a machine and account that did **not** just publish, or from a clean
container with no `CARGO_REGISTRY_TOKEN`, no `UV_PUBLISH_TOKEN`, and no checkout of this
repository. Use a GIL-enabled CPython 3.12 (and again 3.14) and a Rust toolchain ≥ 1.85.

#### crates.io

- [ ] `curl` the crate pages: [fdu](https://crates.io/crates/fdu) and
  [fdu-core](https://crates.io/crates/fdu-core) return 200.

- [ ] Each README renders, and every link on it resolves (no `crates/fdu/`-relative
  404).

- [ ] docs.rs built both: [fdu](https://docs.rs/crate/fdu/0.1.0/builds) and
  [fdu-core](https://docs.rs/crate/fdu-core/0.1.0/builds).

- [ ] From an empty directory:

  ```shell
  cargo install --locked fdu --version 0.1.0
  fdu --version    # fdu 0.1.0
  fdu --help
  fdu --docs
  fdu --cache off .
  ```

- [ ] A new crate depends on the published engine, not a path:

  ```shell
  cargo new --bin fdu-stranger && cd fdu-stranger
  cargo add fdu --locked
  # open("."), print index.total().files, cargo run
  ```

- [ ] `cargo add fdu-core --features watch` builds a consumer that does not pull the
  command-line crate.

#### PyPI

- [ ] https://pypi.org/project/fdu/0.1.0/ lists the source distribution and five wheels,
  and the README shows `uv tool install`, `uvx`, `uv add`, and `pip`.

- [ ] `--no-build` and an explicit GIL interpreter, or a free-threaded default will
  compile the sdist and the check will pass without testing a wheel:

  ```shell
  uv tool run --no-config --no-build --python 3.12 --from fdu==0.1.0 fdu --version
  uv tool run --no-config --no-build --python 3.14 --from fdu==0.1.0 fdu --version
  uvx --no-config --python 3.12 fdu@0.1.0 --help
  ```

- [ ] `pip install fdu==0.1.0` in a 3.12 venv, then `import fdu` and
  `fdu.open(".", cache=fdu.CachePolicy.OFF).total().files`.

- [ ] `index.watch()` accepts one filesystem event and closes.

- [ ] A free-threaded interpreter (`3.14t`) refuses the wheel with a clear error, not a
  hang.

#### GitHub Release

- [ ] https://github.com/jlevy/fdu/releases/tag/v0.1.0 exists, the tag verifies, and
  `gh release view v0.1.0 --json assets --jq '.assets | length'` prints `11` (two
  crates, sdist, five wheels, plus `registry-state.json`, `release-manifest.json`,
  `SHA256SUMS`).
- [ ] The release body is the notes file, not the flowmark-wrapped source with a line
  break in every paragraph.

#### First-Hour Product Checks

From the crates.io or PyPI install, not the checkout:

- [ ] `fdu .` on a small tree matches `fdu . --format json` totals.
- [ ] `fdu . --watch` repaints after creating a file; Ctrl-C returns to the shell.
  On `98379c76` the wheel ignored SIGINT (`fdu-18vk`); pull request #87 restores it.
- [ ] Clone https://github.com/jlevy/fdu.git and run `fdu . --analyze=code` on its
  `crates/` tree with the published binary, not a checkout build.
  The second run reports cached analysis in the performance footer.
- [ ] `fdu --skill` prints a skill that names `fdu`, not a placeholder.

## Testing Strategy

The two phases are the tests.
Phase 1 uses packaged local artifacts and does not contact either registry.
Phase 2 uses published channels from a machine that did not publish.
`make check` does not run either phase: a timing or registry gate on a shared CI runner
measures the runner and the publisher’s credentials.

## Rollout Plan

Publishing is
[Publishing 0.1.0 by Hand](../../guides/release-process.md#publishing-010-by-hand).
This plan does not publish.
Run Phase 1 before that procedure, then Phase 2 from a separate machine after the
channels exist.

## Open Questions

None that block running either phase.
Known product gaps recorded on `98379c76` (`fdu-18vk`, `fdu-i142`) stay on their own
beads; pull request #87 is the engineering leftovers.
Both checklists still record the result.

## References

- [Release process](../../guides/release-process.md)
- [Documentation index](../../../README.md)
- `fdu-yfej`: 0.1.0 first-user stability and usability
- `fdu-bnp9`: pre-publish packaged-artifact simulation
- `fdu-wpxu`: post-publish first-user verification checklist
- `fdu-9cf0`: publish 0.1.0 by hand
- `fdu-18vk`: wheel `--watch` ignores SIGINT (fixed in pull request #87)
- `fdu-i142`: crates.io README relative links (fixed in pull request #87)
- `fdu-8o7g`: whole-millisecond `--interval` (`200ms`)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
