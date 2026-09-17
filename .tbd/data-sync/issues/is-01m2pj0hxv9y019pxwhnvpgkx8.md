---
type: is
id: is-01m2pj0hxv9y019pxwhnvpgkx8
title: "frontmatter-format: YAML 1.1 readers misread plain scalars and NEL folds to a space"
kind: bug
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels:
  - output
  - external
dependencies: []
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:09:29.017Z
updated_at: 2026-09-17T02:11:51.815Z
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
