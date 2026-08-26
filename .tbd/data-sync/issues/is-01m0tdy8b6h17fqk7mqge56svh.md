---
type: is
id: is-01m0tdy8b6h17fqk7mqge56svh
title: Complete the coherent read envelope and version-pinned paging
kind: bug
status: open
priority: 1
version: 35
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5018121437
    at: 2026-08-25T11:04:22.560Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019981640
    at: 2026-08-25T14:15:45.660Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#issuecomment-5412701379
    at: 2026-08-25T15:25:11.905Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5021788526
    at: 2026-08-25T17:13:14.420Z
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
updated_at: 2026-08-26T07:01:50.826Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
At PR #47 exact head e9af881a31243c5c763eff09b2e21ece3a7f5aab, the per-index keyed continuation tag correctly fixes caller-forged counts and the version/shape/error behavior remains good, but the continuation envelope is not yet total. First, EntryCursor::decode refuses token strings longer than 64 KiB while EntryCursor::encode has no matching issuance check and the repository admits longer path encodings (snapshot MAX_PATH_BYTES is 1 MiB; Windows extended paths can exceed the token ceiling after UTF-16 plus hex expansion). The engine can therefore issue a next token it will not accept on the next page. Derive one shared bound from an enforced retained-path limit, or refuse the row/page before issuing; add a maximum-boundary self-roundtrip test and one byte over. Second, public Index derives Clone, copying session and ContinuationAuthority. Two clones can receive different one-commit mutations, reach the same clock with divergent trees, and accept one another’s token because version, shape, session, and MAC all match. Remove raw Index cloning or mint a new session and authority at every independently mutable owner boundary, and test two same-clock divergent clones. Keep this bead open for the broader coherent read envelope as already recorded.

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

--- Round-five findings (e9af881) verified at head d58d9c5. No code changed. ---

The reviewer accepted the keyed-tag portion of this bead. Two findings remain, both
confirmed by reading the code.

1. An issued continuation is not guaranteed to round-trip. encode() emits whatever
   path the cursor retains; decode_inner() refuses more than TOKEN_MAX_CHARS (64 KiB)
   of hex, a ceiling I chose rather than derived. It is unreachable on Linux, where
   PATH_MAX bounds a path well below it, but reachable on Windows: UTF-16 encoding
   doubles the bytes and hex doubles again, so an extended path can exceed 64 KiB
   before hitting its own ceiling. The engine can therefore hand back `next` and then
   refuse that unchanged token. The fix is one bound shared by issuance and decoding,
   or a typed error before issuing.

2. Cloning an Index clones the continuation authority. Index derives Clone at
   index.rs:1559 and ContinuationAuthority derives it too, so two independently
   mutable clones share a key -- and can reach the same clock with divergent trees,
   at which point session, clock, shape and tag all agree and each accepts the
   other's token. Either raw Index cloning goes, or session and authority are minted
   fresh at every independently mutable owner boundary.

Worth recording beside the second: the type's own doc comment says per-open keying is
a scope choice rather than an observable behaviour, on the grounds that the version
check separates two opens first. That reasoning holds for two separately *constructed*
indexes and does not hold for two clones, which is exactly the hole. The comment needs
correcting along with the code.
