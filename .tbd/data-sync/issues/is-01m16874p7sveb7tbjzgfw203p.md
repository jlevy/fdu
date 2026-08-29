---
type: is
id: is-01m16874p7sveb7tbjzgfw203p
title: Align the MetaBrowser contract with the total portable encoding
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-29T07:54:46.342Z
updated_at: 2026-08-29T07:55:04.215Z
---
fdu made the portable encoding total in `13fe8b4`, and MetaBrowser has not followed. Until
it does, the two providers disagree about a value that appears on every row, so any
cross-provider conformance replay diverges on contact.

## The divergence, verified

On MetaBrowser `codex/inventory-contract-alignment`,
`src/metabrowser/inventory_engine/contract.py` still carries:

- `class PortablePathEncoding(StrEnum)` (line 580)
- `class PortablePathIssue` (line 603)
- four `portable_issue: PortablePathIssue | None` fields (lines 996, 1014, 1070, 1111)

fdu deleted all of it: `PortablePathIssue`, `PortablePathExample`,
`PortablePathEncoding`, `MAX_PORTABLE_PATH_EXAMPLES`, `portable_omitted`,
`portable_examples`, and the second completeness flag on `TreePage`.

## What replaces it

Every path has a portable name, so there is no omission to report and no second
population to describe:

- undecodable bytes escape as `%XX` (uppercase hex), and `%` itself escapes as `%25` so
  the mapping stays injective
- valid UTF-8 runs are preserved, so a mostly-readable name stays mostly readable
- `EntryValue.portable_path` is a value, not an `Option`
- `TreePage` has one `complete`, because there is one population; asking twice only
  invited the two answers to diverge
- absence is settled by directory completeness alone, with no "unknown instead of absent"
  branch for a child whose name had no portable form

## Scope

Delete the three types and the four fields; collapse the dual completeness on the tree
page; make the reference provider produce the total encoding so it and fdu agree byte for
byte; update the conformance registry expectations; update the MetaBrowser plan spec so
the two documents state the same rule.

Blocks `fdu-2xfp`: the adapter maps fdu reads onto this contract, so the contract has to
be settled first.
