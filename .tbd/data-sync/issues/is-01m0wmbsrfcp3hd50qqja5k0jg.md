---
type: is
id: is-01m0wmbsrfcp3hd50qqja5k0jg
title: Implement exact MetaBrowser catalog predicate semantics
kind: bug
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019981640
    at: 2026-08-25T14:15:45.882Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:13.550Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T14:14:37.582Z
updated_at: 2026-08-25T16:24:25.976Z
closed_at: 2026-08-25T15:57:00.203Z
close_reason: null
resolution: null
duplicate_of: null
---
At PR 47 exact head 1e9b85d4ce6b4c01fa800f8a25eb607ebb9675a0, the reference catalog_page proves only an unconstrained all-files page. MetaBrowser CatalogQuery also requires case-insensitive terminal-extension matching, exact case-sensitive ancestor-component matching, include_ignored, and an exclusive size upper bound. FDU Selection can exactly translate the ignored tag and exclusive size bound, but its generic case-sensitive path/name globs are not equivalent to the two remaining predicates. Add closed native predicate fields or another demonstrably exact engine-side translation, expose them through Python, and run them through resumable paging and a shared provider/conformance fixture. Do not filter rows in Python or create a mirror index.

## Notes

Shipped as closed native predicates on Selection, applied by the engine.

Three of the contract's five clauses already translated and are now demonstrated
rather than described: files only (Selection.kinds), an exclusive size_less_than as
max_size = size_less_than - 1 in *apparent* bytes (this surface counts allocated
blocks by default, so a tree of small files would otherwise answer a size question
with the block size), and include_ignored=False as the promoted rule's plane, which
is a roll-up read rather than a second walk.

The other two are not globs, in two separate ways, which is why they are closed
fields and not a pattern dialect:

- A terminal suffix is the *last* suffix and is case-folded. `*.md` misses FAQ.MD,
  and a case-insensitive glob dialect would answer this one question by changing
  what every other pattern means. `archive.tar.gz` is `.gz`.
- An ancestor name is a whole component, case-sensitive, any-of. `**/src/**` needs
  one pattern per name, which turns include's any-of into something a caller has to
  reason about.

Selection::terminal_extensions (Vec<String>) and Selection::ancestor_names
(Vec<OsString>), with validating constructors admit_terminal_extension and
admit_ancestor_name on Selection itself -- not in each surface's argument parser,
because a rule stated twice is two rules that agree until one is edited. Every form
that could only ever match nothing is refused where it is written: an undotted
suffix, a bare dot, a compound tail, an uppercase spelling, an empty or multi-
component ancestor name. Reading the consuming contract's own CatalogQuery
__post_init__ showed it validates the identical set, so the two engines refuse the
same values rather than one of them answering with an empty page.

terminal_suffix is transcribed from the contract's rule rather than delegated to
Path::extension. The two agree on this corpus and for different reasons -- Rust's
sheds the dot and returns Some("") for `notes.` and `...` -- so depending on it
would rest an exactness claim on a coincidence of two libraries.

Both axes are path-shaped, which is where the bug would have been. Selection::admits_by_path
is now the one place naming every axis decidable from a path and a name, and
Selection::admits ends in it; watch_session's own filter for removals -- the half of
the stream with no entry to judge -- delegates instead of naming include/exclude by
hand. Three axes have now been forgotten in a hand-written copy of that list. Also
fixed: is_unfiltered did not count the new axes, so a predicate-only query would
have read pre-computed roll-ups and returned unfiltered totals.

Surfaces: --terminal-ext / --ancestor-name on the command line (SELECTION heading,
repeatable), Selection.terminal_extensions / .ancestor_names in Python, both
reaching build_query through the same vocabulary as every other filter. The example's
catalog_page takes all four contract predicates and translates them; it does not
filter rows.

Tests. fdu-core: five unit tests over the rule and the shape, and
a_catalog_predicate_pages_and_resumes_like_any_other_filter in entry_paging (an
uppercase-suffixed file included, four page sizes, exact denominator).
watch_session_integration: a_catalog_predicate_filters_removals_and_not_only_arrivals.
cli: the_catalog_predicates_translate_and_refuse_where_the_library_does. Goldens: two
new sections in cli-axes plus four refusal cases. Python: a new public_smoke check
replaying tests/fixtures/catalog-predicates.json, which is generated by importing the
consuming contract's own CatalogQuery and executing its own matcher over a shared
corpus -- 14 agreement cases, 7 shared refusals, and 2 recorded asymmetries carrying
the other engine's own empty answer, so "the difference is harmless" is a check.

Mutations: 14 against the engine and 6 against the translation, all caught, no
survivors. The six on the translation are the ones worth naming -- allocated instead
of apparent, an inclusive instead of exclusive bound, include_ignored ignored, the
kind clause dropped, and each predicate not passed through.
