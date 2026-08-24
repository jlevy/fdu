---
type: is
id: is-01m0rh6bhzbhf822jt62q14kvn
title: Report views still cannot tell a symlink-only directory from an empty one
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0racd5dxjfx1g5e0dsfay8q
created_at: 2026-08-24T00:02:15.743Z
updated_at: 2026-08-24T17:59:51.978Z
closed_at: 2026-08-24T17:59:51.978Z
close_reason: |
  `others` now rides on `SummaryRow` and `TreeNode`, so the report views can tell a
  directory of symlinks from an empty one — the fact `fdu-5hip` gave the roll-up state but
  the surfaces could not read.

  THE DISPLAY DECISION, which is what this bead was left open to make.

  **A suffix, not a column, and absent rather than zero.** A column would spend width on
  every row of every tree for a number that is zero almost everywhere; a printed `0 others`
  would do the same to the eye. What the fact has to do is stop a hundred symlinks from
  rendering as nothing, and it only has to do that where such a directory exists. So:

      263 B  6 files, 3 directories                 (unchanged, and the common case)
       12 B  0 files, 2 directories, 1 other        (where there is something to say)
       12 B  ██████  100%  links (0 files, 1 other)

  Machine formats carry the field unconditionally, in `summary_json`, the tree node object,
  both YAML shapes, and the binding's dicts. A consumer that had to branch on whether a key
  was present would be a consumer with two code paths for one question, which is the same
  argument the tag work made for always emitting `"tags": []`.

  BOTH TIERS COUNT IT. `summary_from_scalars` reads `RollUpScalars::others` for the
  precomputed tier, and `walk()` tallies non-file, non-directory leaves in its own pre-order
  pass beside the file and directory cases — deliberately not in the post-order fold, for
  the reason already recorded there: the fold sees every directory the walk descended into,
  including ones the selection rejected. A count carried by only one tier is a count the two
  tiers disagree about, so the test asserts they agree.

  TESTS

  - `a_directory_of_symlinks_reports_more_than_an_empty_one` (query_report) pins the summary
    and the tree node in both tiers, and asserts the two directories are not rendered by the
    same triple.
  - `a_directory_of_symlinks_is_rendered_differently_from_an_empty_one` (report_format) pins
    the exact text — singular `1 other`, nothing at all on an empty node — and that JSON and
    YAML carry the zero.

  Not a golden on purpose. Pinning this in the corpus would mean a fixture that creates a
  symlink, and a symlink in a corpus that also runs on Windows is exactly the class of defect
  this branch has already paid for once (an unguarded `symlink_to` in a smoke check, one of
  three Windows-only failures a Linux gate could not see). The renderers are pure functions
  of a report, so a constructed one reaches them exactly and portably.

  `make check` is green. Goldens moved where the machine formats gained the field; the text
  goldens did not move at all, which is the suffix decision working.
resolution: null
duplicate_of: null
---
fdu-5hip added a non-file leaf count to roll-up state, so RollUp, RollUpScalars and a
listing row can now decide emptiness exactly. The report views cannot: SummaryRow and
TreeNode carry files, dirs and bytes, all three of which are zero for a directory holding
only symlinks, so `--view tree` renders it identically to an empty one.

That is the same bug at the CLI surface, and the surfaces are supposed to agree.

Not done in fdu-5hip on purpose: adding a column to the text table is a display decision
about the command line rather than an engine one, and it moves every text and JSON
golden. Worth doing deliberately, with the column's name and placement chosen rather than
inherited.

WHAT IT NEEDS: `others` on SummaryRow and TreeNode, carried through
summary_from_scalars and the tree builder, emitted in the JSON and YAML shapes, and a
decision about the text views -- a column, a suffix on the count, or nothing at all with
the fact reachable only in machine formats. Goldens follow whichever is chosen.
