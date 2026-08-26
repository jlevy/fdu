---
type: is
id: is-01m0wmbsrfcp3hd50qqja5k0jg
title: Implement exact MetaBrowser catalog predicate semantics
kind: bug
status: open
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019981640
    at: 2026-08-25T14:15:45.882Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:13.550Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5021788526
    at: 2026-08-25T17:13:14.178Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5025612603
    at: 2026-08-26T00:28:56.520Z
labels:
  - pr47-review
  - metabrowser
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T14:14:37.582Z
updated_at: 2026-08-26T00:28:56.523Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
At PR #47 exact head 0558c7eff1b91a1dca052d4259dbe3751f6ffcd0 versus MetaBrowser #74 exact head 0577bb125c4a607719befa3f213362f5522d5724, catalog predicates are still not a runtime-independent exact contract. The new FDU spec rejects the prior ..foo finding after checking CPython 3.11-3.13, but MetaBrowser supports 3.12-3.14. Verified with uv: PurePosixPath("..foo").suffix is ".foo" on 3.12/3.13 and "" on 3.14; PurePosixPath("foo.").suffix changes from "" to ".". MetaBrowser's Python provider therefore changes answers by supported runtime. FDU's explicit terminal_suffix is stable and matches 3.12/3.13. Align both owned sides on one stated rule, preferably the existing FDU rule (last dot, neither first nor final: ".gitignore" and "foo." have none, "..foo" has ".foo"), and replace MetaBrowser's pathlib dependency with the contract helper tracked by mb-5npa. Remaining FDU work: reject duplicate terminal_extensions and ancestor_names; reject both separators in ancestor names on every platform. MetaBrowser should reject "." and ".." ancestor names so queries that canonical paths can never match fail at construction, matching FDU. Turn every boundary into a shared acceptance/refusal case across Python 3.12/3.13/3.14 and the Rust/Python binding: ..foo, .foo, foo., duplicates, dot components, and a real POSIX backslash directory. The generated fixture must not silently depend on the interpreter used to regenerate it. No adapter-side filtering, runtime-version branch, or mirror state.

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

Reopened: Reopened after exact-head review of e9af881: the shipped predicate and fixture claim exact MetaBrowser agreement but retain observable answer and validation differences. PurePosixPath("..foo").suffix is empty while FDU terminal_suffix("..foo") returns .foo; FDU accepts a backslash-containing ancestor on POSIX although CatalogQuery rejects it and such a directory can exist; and FDU accepts duplicate terminal/ancestor entries that CatalogQuery rejects. The fixture omits the answer-changing cases and explicitly blesses asymmetries.

--- Verified at head d58d9c5, in response to the e9af881 reopen. No code changed. ---

One premise of the reopen does not hold, and it is the headline one.
PurePosixPath("..foo").suffix is ".foo", not empty -- checked on CPython 3.11, 3.12
and 3.13. fdu's terminal_suffix("..foo") also returns ".foo", and the unit test
asserts that. The two engines agree on this name; there is nothing to fix here, and
changing terminal_suffix to return empty would introduce the disagreement the finding
describes.

The other two points are real, and one of them corrects an overstatement of mine.

- Duplicate entries. CatalogQuery.__post_init__ rejects duplicates in both lists;
  admit_terminal_extension and admit_ancestor_name append them. Harmless to the answer
  (both lists are any-of, so a duplicate narrows nothing) but a validation difference,
  and cheap to close.
- A backslash in an ancestor name. CatalogQuery rejects it on every platform;
  Component::Normal admits it on POSIX, where it is a legal directory name. I called
  this asymmetry harmless in the PR comment and that was overstated: the fixture only
  asserted an empty result over a corpus containing no such directory, which is a
  weaker check than the claim. The consumer refuses the query outright, so no answer
  either engine returns is wrong -- but a caller can construct the case, and the
  fixture does not cover it.

The reopen's criticism of the fixture is fair on its own terms: a fixture that records
tolerated asymmetries cannot by itself close an exactness gate. Closing it means
deciding which contract moves, and the reviewer suggests tightening the consumer to
reject "." and ".." -- a cross-repo decision, not one to make from this side.
