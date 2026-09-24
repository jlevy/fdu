# fdu

**Fast, incremental file roll-up engine** — `fd` and `du`, read as “fast du”.

fdu answers, for *every* directory in a tree at once: how big is it, how many files does
it hold, what changed most recently, and what kinds of files live in it.
One walk, many metrics, with reusable metadata and content state.

This crate is the `fdu` command line.
It is also a library that re-exports the whole
[`fdu-core`](https://crates.io/crates/fdu-core) engine, so there is one name to know for
installing the tool and for depending on it.

**Status: 0.x.** A new minor release may change the Rust API, the Python API, or the
command line;
[the release process](https://github.com/jlevy/fdu/blob/main/docs/project/guides/release-process.md)
states the compatibility rules.

## Install

Install the command line with Rust 1.85 or newer:

```shell
cargo install --locked fdu
fdu --help
```

`--locked` builds against the `Cargo.lock` published with the crate.
Without it Cargo re-resolves every dependency to the newest compatible release, which
bypasses the review and release cool-off
[the supply-chain policy](https://github.com/jlevy/fdu/blob/main/SUPPLY-CHAIN-SECURITY.md)
applies to the dependency set.

The [`fdu` Python package](https://pypi.org/project/fdu/) carries the same command line
in prebuilt wheels, so `uv tool install fdu` installs it without a Rust toolchain,
`uv tool upgrade fdu` updates it, and `uvx fdu@latest` runs the latest release without
installing anything.

For coding agents, `fdu --install-skill` writes the agent skill to
`.agents/skills/fdu/SKILL.md` and `.claude/skills/fdu/SKILL.md` under the project root
(`--agent-base DIR` for one agent’s user scope, such as `~/.claude`), and `fdu --skill`
prints it.

## Use

fdu requires a path; bare `fdu` prints help and scans nothing.

```shell
fdu .                                     # directory tree: allocated sizes, largest first
fdu . --view=summary                      # one total for the tree
fdu . --view=languages                    # which languages occupy space
fdu . --view=recent --limit=10            # the ten most recently modified files
fdu . --exclude-ignored                   # leave out entries .gitignore rules match
fdu . --analyze=code                      # standard lines of code; reads file contents
fdu . --view=summary,types --format=json  # versioned machine output
fdu --docs                                # the offline usage guide
```

Without `--analyze`, fdu reads metadata and `.gitignore` files but never opens a regular
file for its contents.
`--view` chooses what is reported and never enables analysis.

## As a Library

```shell
cargo add fdu
```

The default `watch` build feature adds the OS-native watch layer;
`cargo add fdu --no-default-features` leaves it out.
The command line’s own dependencies come with `fdu` either way, so an embedding that
wants none of them depends on [`fdu-core`](https://crates.io/crates/fdu-core) instead.
The API is documented on [docs.rs](https://docs.rs/fdu), and
[the repository README](https://github.com/jlevy/fdu#as-a-rust-library) has examples.

## Documentation

- [Usage guide](https://github.com/jlevy/fdu/blob/main/docs/usage.md): every view,
  analyzer, cache policy, selection, and automation contract
- [Repository README](https://github.com/jlevy/fdu#readme): performance evidence, cost
  layers, and how the engine works
- [0.1.0 release notes](https://github.com/jlevy/fdu/blob/main/docs/project/release-notes/0.1.0.md)
  and [changelog](https://github.com/jlevy/fdu/blob/main/CHANGELOG.md)
- [Security policy](https://github.com/jlevy/fdu/blob/main/SECURITY.md)

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
