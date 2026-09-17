# YAML Scalar Conformance

A corpus of 121 strings that must survive a YAML round trip as the same string, and a
matrix that emits each one as `{path: <string>}` and loads it back with six parsers.
It is the evidence behind the YAML scalar policy in
[the explicit core models plan](../../docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md)
(bead `fdu-4xy9`), and the corpus the answer model’s `emit` tests copy.

## What It Covers

`corpus.json` holds the strings: numbers in every base YAML resolves (`0x10`, `1_000`,
`0b101`, `0o17`, `.inf`, `.NaN`), YAML 1.1 booleans (`y`, `on`), sexagesimal times,
dates, indicator characters, quotes, leading and trailing space, line breaks, tabs, DEL,
C1 controls including NEL, U+2028 and U+2029, a byte-order mark, and U+FFFE.

The emitters are fdu’s release-candidate and branch rules re-implemented in Python
(`py_emit.py`), the prototype policies (`proto_emit.py`, including a fixed
`frontmatter-format` string representer), PyYAML and ruamel dumps, and npm `yaml`
`stringify` (`node_parse.mjs`). The parsers are PyYAML (pure and libyaml, YAML 1.1),
ruamel.yaml (safe and round-trip, YAML 1.2), and npm `yaml` in strict 1.1 and 1.2 modes.
Rust YAML crates were evaluated separately and are summarized in the results.

## Running It

From this directory, with the parser versions the evaluation used pinned and npm `yaml`
from the repository’s `node_modules`:

```shell
uv run --no-project --python 3.12 --with pyyaml==6.0.3 --with ruamel.yaml==0.19.1 python py_emit.py
uv run --no-project --python 3.12 --with ruamel.yaml==0.19.1 python proto_emit.py
node node_parse.mjs
uv run --no-project --python 3.12 --with pyyaml==6.0.3 --with ruamel.yaml==0.19.1 python py_parse.py
```

The scripts write `emit-*.tsv`, `results-py.json`, and `results-node.json` here, ignored
by Git.

## Results

[`results/2026-09-17-summary.txt`](results/2026-09-17-summary.txt) counts, per emitter
and parser, the strings misread or rejected, and lists each failure.
No standard emitter quoted correctly for both YAML versions; the proposed strict policy
and the fixed `frontmatter-format` prototype had no failures.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
