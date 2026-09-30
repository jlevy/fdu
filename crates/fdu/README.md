# fdu

**Fast native du replacement and detailed file analytics for Python and Rust**

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

## Set Up with Any Coding Agent

Give the agent this instruction:

> Run `uvx --no-build fdu@latest --install-skill` from the project root to install fdu’s
> self-contained skill for current and future agent sessions.

The installer writes `.agents/skills/fdu/SKILL.md` and `.claude/skills/fdu/SKILL.md`.
The skill needs no prior session context; `fdu --skill` prints it.

## Install the Command Line

With [uv](https://docs.astral.sh/uv/), run the prebuilt command without a Rust toolchain
or a persistent install, or keep it on your `PATH`:

```shell
uvx --no-build fdu@latest .
uv tool install --no-build fdu
fdu .
```

`--no-build` requires a compatible wheel and fails instead of compiling from source.
The [Python package](https://pypi.org/project/fdu/) publishes wheels for GIL-enabled
CPython 3.12 and newer on Linux glibc (x86-64 and arm64), macOS (x86-64 and arm64), and
Windows x86-64. No Python version is needed in normal use.
If uv selects free-threaded CPython, such as `3.14t`, retry with `--python 3.14`.
`uv tool upgrade fdu` updates a persistent install.

To install the Rust crate from source, use Rust 1.85 or newer:

```shell
cargo install --locked fdu
fdu --help
```

`--locked` builds against the `Cargo.lock` published with the crate.
Without it Cargo re-resolves every dependency to the newest compatible release, which
bypasses the review and release cool-off
[the supply-chain policy](https://github.com/jlevy/fdu/blob/main/SUPPLY-CHAIN-SECURITY.md)
applies to the dependency set.

## Use

fdu requires a path; bare `fdu` prints help and scans nothing.

```shell
fdu .                                     # directory tree: allocated sizes, largest first
fdu . --view=summary                      # one total for the tree
fdu . --view=languages                    # which languages occupy space
fdu . --view=recent --limit=10            # the ten most recently modified files
fdu . --ignored=exclude                   # leave out entries .gitignore rules match
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
- [0.3.0 release notes](https://github.com/jlevy/fdu/blob/main/docs/project/release-notes/0.3.0.md)
  and [changelog](https://github.com/jlevy/fdu/blob/main/CHANGELOG.md)
- [Security policy](https://github.com/jlevy/fdu/blob/main/SECURITY.md)

License: MIT.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
