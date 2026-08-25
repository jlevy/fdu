---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 29
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5018121437
    at: 2026-08-25T11:04:22.560Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019981640
    at: 2026-08-25T14:15:45.660Z
labels:
  - pr47-review
  - metabrowser
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0tdy9ceep2byvbtyvwc2vky
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T17:43:53.445Z
updated_at: 2026-08-25T15:56:36.154Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
At PR 47 head e658915, the core ReadBundle captures clock, scope, freshness, and projections under one guard, but PyIndex.read releases that guard and then locks RunState to attach complete, source, and errors. A refresh can therefore pair old data with new status or new data with old status. ReadRequest also has no requested clock or version, so a multi-page catalog can silently mix states after a mutation. Fix: return lifecycle, coverage, freshness, source, progress, and typed issues from the same versioned engine image; add an expected session and clock to a read and return VersionUnavailable on mismatch. A provider may retain only the current version: page two either sees the exact version or fails, never advances silently. Add forced interleaving and mutation-between-pages tests. This is follow-up to closed fdu-2ivi and should precede the wider algebra in fdu-samw. Review finding FDU47-R4.

## Notes

Answered, at branch head after the token change (F1/F2 of the fourth review round).

EntryCursor has private fields and no constructor. A caller cannot write one, so
the counts it carries can only be counts the engine established by walking. It
crosses a language boundary as EntryCursor::encode / EntryCursor::decode -- hex,
with an FNV checksum over a magic-tagged payload -- because the accident that a
public struct invites is a value round-tripped through a wire format that dropped
or reordered a field, arriving as an honest-looking claim about an answer nobody
computed. The Python facade emits `next` as that opaque string and takes it back
unchanged; there is no adapter state and no second cursor.

The continuation is bound to the question it was issued for. EntryPageRequest::shape
mixes the root, the depth bound, the plane and Selection::shape -- an explicit FNV
over every result-shaping field rather than a derive, so adding a field and
forgetting the shape is a compile-visible omission rather than a silent collision.
Resuming under a different question is Error::InvalidValue{kind: "page continuation"},
never an answer carrying the first question's denominator. Two fields are
deliberately outside the shape and say so: the page `limit`, so a caller may change
page size mid-assembly, and Selection::size, which chooses which number a report
renders rather than which entries a page admits.

entry_page now returns crate::Result<Option<EntryPage>>. A version mismatch is
Error::VersionUnavailable whether or not the caller also pinned with
ReadRequest::expected -- the None-through-Option path is gone -- and impossible
arithmetic is an error rather than a saturating subtraction that would report a
plausible remainder.

Tests: a_continuation_belongs_to_one_question_and_is_refused_by_any_other (six
edits, now including a terminal suffix and an ancestor name),
a_tampered_token_is_refused, a_continuation_from_another_version_is_refused, and
the existing flat-continuation and one-denominator tests unchanged. The Python
smoke asserts the same across the binding, including that a path, an empty string,
a truncated token and a single flipped character are all refused as continuations.

Left open deliberately: this bead also carries the envelope work, and the reviewer
reopened it. Not re-closing.
