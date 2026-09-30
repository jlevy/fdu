---
type: is
id: is-01m2pj0hxv9y019pxwhnvpgkx8
title: "frontmatter-format: YAML 1.1 readers misread plain scalars and NEL folds to a space"
kind: bug
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - external
dependencies: []
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:09:29.017Z
updated_at: 2026-09-30T10:10:11.438Z
closed_at: 2026-09-30T10:10:11.437Z
close_reason: |
  Not fdu work; closed here, belongs in github.com/jlevy/frontmatter-format. The defect is in that package's string representer (represent_str in frontmatter_format.yaml_util), a separate first-party PyPI package. `attic/` is this repository's gitignored directory of third-party reference checkouts (.gitignore: "Third-party source checked out for reference only, never built or shipped"), so attic/frontmatter-format is a local clone of that project, not part of fdu. fdu neither ships nor imports frontmatter-format: it appears only transitively through softschema in the benchmark/docs tool environment (explorations/benchmarks/uv.lock), as a first-party exclude-newer entry, and in the explorations/yaml-conformance prototypes. The proposed fix and its passing prototype are in explorations/yaml-conformance/proto_emit.py and this bead's notes; file them upstream (sidematter-format and metabrowser inherit the fix). fdu's own YAML direction stays with fdu-omo5.
resolution: canceled
duplicate_of: null
---
In jlevy/frontmatter-format (attic/frontmatter-format), ruamel.yaml (YAML 1.2) dumps `on` and `12:30:00`
plain; PyYAML (YAML 1.1) reads them back as True and 45000. A string containing NEL (U+0085) is emitted
raw in single quotes and loads back with the NEL replaced by a space, even through ruamel itself.
Candidate fix: a string representer that quotes scalars any 1.1 or 1.2 resolver would not read as a
string and double-quotes with escapes for C1 controls; test with the shared corpus. Work belongs in that
repository; tracked here for the cross-project YAML direction.

## Notes

2026-09-17 evaluation detail. frontmatter-format misreads 19 of 121 corpus strings in at least one
parser. Proposed fix in represent_str (~30 lines, keep ruamel): use `|` only for block-safe text (no CR,
NEL, LS, PS, C0/C1, DEL, BOM, tabs, whitespace-only lines, or leading space on the first line; `x\r\ny` as
`|-` loads as `x\ny` everywhere); force double quotes when ruamel's VersionedResolver (1,1) or (1,2)
resolves the text as non-string, plus the 1.1 float pattern, `=`, `<<`, y/n; force double quotes for any
NEL, LS, PS, C1, DEL or BOM, which also sidesteps ruamel's single-quoted emitter writing NEL as a raw break
(worth reporting upstream). Prototype passed the full matrix. sidematter-format and metabrowser inherit it.
