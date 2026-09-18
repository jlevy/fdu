# First-Release Verification

Two checklists for the first public `0.1.0`. The first can be run before either registry
exists; the second can run only after
[Publishing 0.1.0 by Hand](release-process.md#publishing-010-by-hand) finishes.

## Before the Registries Exist

These steps use only packaged local artifacts: `cargo package` for both crates, the host
`abi3` wheel, and the source distribution.
They do not contact crates.io or PyPI. `make release-rehearse` already packages and
inspects those artifacts; [the new-user simulation](#new-user-simulation) is the
stranger path on top of them.

A 2026-09-18 run against `origin/main` (`98379c76`) is recorded on `fdu-bnp9`.

### New-User Simulation

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

3. **Watch.** Start `fdu --watch --view files --format jsonl --cache off TREE`, create a
   file in `TREE`, and confirm a JSONL envelope arrives.
   Interrupt with Ctrl-C. A wheel-installed command that ignores Ctrl-C during `--watch`
   is `fdu-18vk`.

4. **Python, from the host wheel, not a rebuild:**

   ```shell
   uv tool install --no-index --from ./fdu-0.1.0-*.whl --python 3.12 fdu
   uv run --python 3.12 -c 'import fdu; print(fdu.open(".", cache=fdu.CachePolicy.OFF).total().files)'
   ```

   Confirm `uv tool run --from that.whl fdu --version` is the same binary identity as
   the crate install.

5. **Rust consumer** against the path crates (`cargo add` cannot resolve `fdu` until
   crates.io has it): `open` a tree, print `index.total().files`.

6. **Registry-page rehearsal.** `crates/fdu/Cargo.toml` `readme` and `crates/fdu-py`
   `[project.urls]` decide what a stranger sees on crates.io and PyPI. Relative links in
   a README that crates.io resolves under `crates/fdu/` will 404. That defect is
   `fdu-i142` / pull request #77.

## After 0.1.0 Is on the Channels

Run these from a machine and account that did **not** just publish, or from a clean
container with no `CARGO_REGISTRY_TOKEN`, no `UV_PUBLISH_TOKEN`, and no checkout of this
repository. Use a GIL-enabled CPython 3.12 (and again 3.14) and a Rust toolchain ≥ 1.85.

### crates.io

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

### PyPI

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

### GitHub Release

- [ ] https://github.com/jlevy/fdu/releases/tag/v0.1.0 exists, the tag verifies, and
  `gh release view v0.1.0 --json assets --jq '.assets | length'` prints `11` (two
  crates, sdist, five wheels, plus `registry-state.json`, `release-manifest.json`,
  `SHA256SUMS`).
- [ ] The release body is the notes file, not the flowmark-wrapped source with a line
  break in every paragraph.

### First-Hour Product Checks

From the crates.io or PyPI install, not the checkout:

- [ ] `fdu .` on a small tree matches `fdu . --format json` totals.
- [ ] `fdu . --watch` repaints after creating a file; Ctrl-C returns to the shell
  (`fdu-18vk` is the known wheel gap).
- [ ] `fdu . --analyze=code` on this repository’s `crates/` tree completes and the
  second run reports cached analysis in the performance footer.
- [ ] `fdu --skill` prints a skill that names `fdu`, not a placeholder.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
