---
type: is
id: is-01m16874p7sveb7tbjzgfw203p
title: Align the MetaBrowser contract with the total portable encoding
kind: task
status: in_progress
priority: 1
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-29T07:54:46.342Z
updated_at: 2026-08-30T17:11:05.822Z
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

## Notes

## Landed: `c585f20` and `a5f5e55` on `codex/inventory-contract-alignment`

The contract states the total encoding, produces it, orders by it, and no longer carries
the machinery a partial encoding needed. `make verify` green: 1635 tests, 48 CLI goldens.

## Resolved: one path, not two

The fork this bead left open is settled, and settled more cheaply than the fdu-mirroring
option suggested. fdu does not store two paths either -- its arena holds the native path
and derives `portable_path` per returned row, so the second form exists for a page, never
for the index. Mirroring that here means escaping at `_semantic_entry`, the single
outbound boundary, and leaving `FsEntry` holding exactly what the filesystem gave it.

No entry holds two strings. That matters because the two forms are equal for essentially
every file: only an undecodable byte or a literal `%` makes them differ, so storing both
would be a cost paid by the whole corpus to describe a case most trees never contain.

The common path is C-level -- a `%` substring scan plus an `encode` that raises precisely
on the surrogates marking an undecodable byte, rather than a Python scan over every
character of every name in every page. 99ns per call, returning the same object, so an
ordinary name allocates nothing. A test pins the identity.

Pinning it found real waste: `canonical_inventory_path` split on `/` and rejoined
unconditionally, allocating for every path on every page including the untouched ones. No
escape rule produces or consumes `/`, so it is now the same function as the single-name
one and inherits the fast path.

## Deferred, on purpose

`fdu-jng6`: about seven sites use a contract path as a filesystem address, so an escaped
entry is listable, orderable, and consistent across providers, and may fail to open.
Before this work one undecodable name made the whole directory unlistable, so this is
strictly better, and the inverse is available whenever someone needs it because the
encoding is injective.
