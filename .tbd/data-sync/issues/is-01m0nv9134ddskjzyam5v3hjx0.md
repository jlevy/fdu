---
type: is
id: is-01m0nv9134ddskjzyam5v3hjx0
title: Decide the published crate names before fdu ships to crates.io
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-22T23:00:45.795Z
updated_at: 2026-08-22T23:00:45.795Z
---
Moving the command line into crates/fdu-cli changed what a user types to install it, and that decision should be made deliberately rather than inherited from a refactor.

Today: A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples. is the library and publishes no binary;  carries the binary, which is still named A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples.. So the install command becomes  while the program is A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples..

That is a common Rust layout and it is defensible, but it is surprising: a user who knows the tool is called fdu will type  and get a library with no binary, which is exactly the error the README used to produce.

Two ends, both coherent:

1. Keep it. A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples. is the library,  installs the command. Document it once, in the README and in the release notes, and accept that  fails with cargo's own 'has no binaries' message -- which is at least a clear error rather than a wrong result.

2. Swap the names. The binary crate takes A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples. and the library becomes  or similar.  then works as a user expects. Costs a library rename before anything is published, which is the cheapest moment it will ever be.

Nothing is published yet, so this is free to decide now and expensive later: crates.io names are permanent and a published A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples. library cannot become a binary crate.

Also update scripts/release/registry_state.py, which checks crates.io for A fast, incremental file roll-up engine: hierarchical tallies over large directory trees

Usage: fdu [OPTIONS] <PATH>
       fdu [PATH] --cache-status[=<SCOPE>] [--cache-clear[=<SCOPE>]]
       fdu [PATH] --cache-clear[=<SCOPE>]
       fdu --docs
       fdu --skill

ARGUMENTS
  [PATH]  Report root; optional only for the discovery and cache-lifecycle flags

SCOPE
      --scan-depth <N>  Limit scanning and retention to N entry levels
      --one-filesystem  Stay on the filesystem the root lives on

SELECTION
      --include <GLOB>          Report only entries matching this glob; repeatable
      --exclude <GLOB>          Exclude entries matching this glob; repeatable, and wins over
                                --include
      --min-size <SIZE>         Report only entries at least this large, as 512, 10M, or 1.5GiB
      --modified-since <WHEN>   Report only entries modified at or after this time, as 2h or an RFC
                                3339 stamp
      --modified-before <WHEN>  Report only entries modified before this time
      --kind <LIST>             Entry kinds to report: file, dir, symlink, other
  -d, --depth <N>               Directory levels to show; does not limit scanning. Accepts `all`
                                [default: 2]
  -n, --limit <N>               Rows to show, per group. Accepts `all`
      --sort <KEY>              Order results: size, count, mtime, or name
      --reverse                 Reverse the ordering
      --size <METRIC>           Which size metric to report: allocated or apparent [default:
                                allocated]

VIEWS
      --view <LIST>         Views: tree, extensions, types, families, languages, documents, largest,
                            recent, files, summary, or full. Defaults to the view that displays what
                            --analyze asked for
      --words-per-page <N>  Logical words per derived document page [default: 250]

CONTENT ANALYSIS
      --analyze <LIST>        Analyzers to run: none, lines, code, words, or all [default: none]
      --analysis-workers <N>  Content reader workers; zero selects available parallelism [default:
                              0]

OUTPUT
      --format <FORMAT>  Output format: text, json, jsonl, or yaml [default: text]
      --color <WHEN>     Colorize human output: auto, always, or never [default: auto]

EXECUTION
      --cache <POLICY>  Cache policy: auto, refresh, read-only, only, or off [default: auto]
      --allow-partial   Accept operationally partial results, including filesystem or analysis
                        failures
      --watch           Stream changes continuously instead of returning one report
      --interval <DUR>  How often aggregate views re-render while watching, as a duration [default:
                        2s]

CACHE MANAGEMENT
      --cache-status[=<SCOPE>]  Report cache contents instead of scanning: root (default) or all
      --cache-clear[=<SCOPE>]   Remove cached snapshots instead of scanning: root (default) or all

OTHER
  -h, --help     Print help
  -V, --version  Print version
      --docs     Print the usage guide: the report ladder, both axes, and the output contracts
      --skill    Print a portable agent skill to stdout

Run `fdu --docs` for more help and important usage examples. alone and would not notice fdu-cli missing, and the release packaging spec, whose goal line says 'make cargo install fdu ... expose the same command-line contract'.
