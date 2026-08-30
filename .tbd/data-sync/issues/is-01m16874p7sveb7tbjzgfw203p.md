---
type: is
id: is-01m16874p7sveb7tbjzgfw203p
title: Align the MetaBrowser contract with the total portable encoding
kind: task
status: open
priority: 1
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
created_at: 2026-08-29T07:54:46.342Z
updated_at: 2026-08-30T07:46:13.869Z
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

## Found while aligning: the reference provider crashed on undecodable names

Not merely a type mismatch. `_child_order` ordered by `row.name.encode("utf-8")`, and
`os.scandir` decodes an undecodable byte with `surrogateescape`, so the name holds a
scalar in `U+DC80..U+DCFF` and encoding it raises `UnicodeEncodeError`. One such file made
the entire directory unlistable, where fdu escaped it and listed it. The catalog order key
had the same shape.

The recency keys diverged differently: they sorted by `str` (code point order) while the
other two sorted by encoded bytes. Those agree for valid UTF-8, since UTF-8 preserves code
point order, and stop agreeing exactly where surrogates appear.

## Landed on `codex/inventory-contract-alignment`

- the contract states the total encoding: `%XX` uppercase for undecodable bytes, `%25` for
  `%` so the mapping stays injective, valid UTF-8 runs preserved, and it is a name rather
  than an address
- `canonical_inventory_name` / `canonical_inventory_path` produce it, with the platform
  branch mirroring fdu: single escape per surrogateescape byte on POSIX, two big-endian
  escapes per unpaired UTF-16 surrogate on Windows
- all four order keys canonicalize, so ordering is total and matches fdu byte for byte
- `require_canonical_inventory_path` now rejects surrogates, so a raw platform name is
  refused at the boundary rather than accepted and ordered inconsistently later
- `PortablePathEncoding`, `PortablePathExample`, `PortablePathIssue`, the two example
  bounds, and the four `portable_issue` fields are gone, along with the spike adapter's
  omission guards and the architecture doc's description of them

Verified against fdu's output: `x\xff.txt` becomes `x%FF.txt` in both, `100%.txt` becomes
`100%25.txt`, and the escaped form is encodable where the raw one raises.

## Remaining, and it is now forced rather than optional

`InventoryEntry.__post_init__` validates `path` and `parent` with the tightened checker,
so constructing a row from a raw platform name now raises at construction instead of later
at ordering. Both are failures; the fix is for the provider to canonicalize before it
builds a contract row.

That settles the design fork this bead left open. MetaBrowser's contract row is the
analogue of fdu's `portable_path`, not of fdu's native `path`: it is the consumer-facing
identity, and the provider's own `FsEntry` store is the analogue of fdu's arena and can
stay native. So canonicalize at `_semantic_entry`, the boundary that already exists to
drop engine bookkeeping, rather than adding a second path field to every projection.

Open question for that step: whether the store should also be keyed by the canonical form,
since a query names a path the client got from an earlier row. fdu keys its child maps by
the portable component for exactly this reason.
