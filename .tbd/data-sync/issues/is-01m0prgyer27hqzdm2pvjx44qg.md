---
type: is
id: is-01m0prgyer27hqzdm2pvjx44qg
title: "Partitioned tallies: tag rules and per-plane roll-ups in the engine"
kind: feature
status: open
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
  - type: blocks
    target: is-01m0ptezmtmkn04mh1f1rwgdxb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:53.944Z
updated_at: 2026-08-24T08:44:20.643Z
---
Opt-in tag configuration on ScanOptions: compiled gitignore (ignore-crate matcher, correct negation) and hidden-with-allowlist rules; entry tag bits; per-plane roll-up state (files, dirs, bytes, allocated, newest mtime, per-extension) through merge_upward, refresh, and watch re-tagging; .gitignore-edit escalation to InvalidateSubtree; enabled rule set versions the snapshot fingerprint. Builds on the closed spike fdu-p35d (0.39-1.76 us/entry measured). Partition-sum property tests and fingerprint-invalidation tests land with it.

## Notes

SCOPE SETTLED — the earlier "awaiting metabrowser confirmation" note is stale and the
answer came back smaller than the bead's description.

The reconciliation (research-2026-08-23-interactive-contract-reconciliation.md) and
metabrowser's reply on PR #44 resolve it: hidden paths PRUNE AT SCOPE with an exact-name
allowlist, they do not become a second maintained tag plane. The plane had no consumer
once the product had no hidden toggle, and a tag plane would still have to walk the
hidden trees it exists to exclude. The third-plane open question closes as no.

So GITIGNORE IS THE SOLE TAG RULE here, and this bead is two pieces of work rather than
three: the gitignore matcher plus the unignored plane through the reducer path, and
hidden admission as a scope rule fingerprinted into snapshot identity.

WHAT ACTUALLY BLOCKS IT NOW: one undecided design question, not a queue position. The
cool-off framing was wrong -- it applies to a release being younger than 14 days, and
`ignore` is mature, so picking a settled release is a chore. The real question is
dependency weight on fdu-core, MEASURED 2026-08-24 rather than estimated:

  9 new crates: ignore, globset, aho-corasick, bstr, regex-automata, regex-syntax,
  crossbeam-deque, crossbeam-epoch, crossbeam-utils. Lockfile 73 -> 82. It pulls
  regex-automata + regex-syntax directly, not the `regex` facade crate.

  Binary size +1.06 MiB. Measured under fdu's own release profile (opt-level 3, lto,
  codegen-units 1, strip) with a REALISTIC use -- build a GitignoreBuilder from rules and
  match a path -- because an unused import is stripped and would understate it. Empty
  binary 312 KB; the same binary doing real gitignore work 1393 KB. fdu is 3.45 MiB
  today, so about +29%.

  Cold compile of that subtree ~13s against fdu's ~59s full release build. Noisy host.

  No lean mode exists: `ignore` has one feature (simd-accel) and globset's only trimmable
  default is `log`. regex-automata is mandatory. Whole or nothing.

FOR CONTEXT, what comparable tools do. ripgrep is NOT a precedent -- it owns ignore and
globset as in-repo workspace members and publishes them. fd is the precedent: it takes
ignore, globset, regex, regex-syntax, aho-corasick and crossbeam-channel among 16 direct
dependencies, and nobody calls fd bloated. At 4.5 MiB fdu's binary is unremarkable for
its class.

THE ASYMMETRY THAT ACTUALLY MATTERS: fd and ripgrep are binaries, where dependency weight
is a distribution question. fdu-core is a LIBRARY and imposes it on every consumer --
the Python wheel, and any Rust caller wanting disk totals with no interest in gitignore.

PRECEDENT ALREADY IN THIS REPO: notify is feature-gated exactly this way. fdu-core has
default = ["watch"], whose comment reads "the shipped binary watches;
--no-default-features and library consumers do not", and CI tests -p fdu-core
--no-default-features AND --features watch separately. Parity is unaffected: it compares
the built CLI against the built wheel, both with default features on, while the
feature-boundary job only checks that the crate still compiles without them. So
default = ["watch", "gitignore"] gets the correct matcher and keeps the core lean, with
no new machinery invented for it.

Awaiting the owner's call; do not decide it unilaterally.

The spike fdu-p35d already measured the matcher itself at 0.39-1.76 us/entry and closed.

READ BEFORE BUILDING: fdu-n4gn prices this plane together with groups, composed
provenance and leaf counts as ONE reducer union, on the ancestor-merge path exp-064 took
from 43.73% to 14.07% and campaign 2 plans to delete rather than tune. A cost acceptable
for each alone can be wrong in combination, and leaf counts have already shipped
unpriced (fdu-2ig2).

GENERICITY REVIEW 2026-08-24 — PROPOSAL, NOT APPLIED. Raised by the owner: gitignore
should be one flag among several (text, binary, mime type, ...), not the feature's name,
so the code does not fill up with if-gitignore blocks. Checked the code and the spec.

NOTHING IS BUILT. Zero tag or plane code in the engine, CLI or binding. The only artifact
is the reserved slot ScanScope.ignore_rules_fingerprint, value 0, comment "No ignore rules
exist yet", serialized positionally into the snapshot header. That makes this the cheapest
possible moment to fix the shape, and a Rust field rename does not touch the wire format.

THE SPEC IS ALREADY MOSTLY GENERIC: "tags are observations; planes are maintained
aggregates", entries carry tag BITS, an enabled rule SET, --tags as scope and --plane as a
selection axis taking a value defaulting to `all`, partition property stated per tag, rule
set versions the fingerprint. gitignore is named as the first rule enabled, not as the
mechanism. Roughly 70% of the way there.

THREE THINGS THAT ARE NOT GENERIC:

1. TAGS AND PLANES ARE 1:1, and it has already deformed the design once. The spec says
   "for each enabled tag, every directory's roll-up maintains one additional plane", so a
   second tag doubles maintained state again. Hidden was DEMOTED from a tag to a
   scope-prune rule, and one stated reason is cost -- "a plane would still have to walk
   .git, caches, and virtualenvs". That is not hypothetical: the coupling already forced
   one axis out of the model.

   Many tags is cheap (a bit in an entry record). Many planes is not (the reducer union on
   the ancestor-merge path, which fdu-n4gn prices and campaign 2 plans to delete rather
   than tune). Decoupling -- unbounded tags, a small DECLARED promoted subset -- costs
   nothing to state now, because the two-tier query rule already in the spec handles an
   unpromoted tag by filtering.

2. NO TIER ON A TAG RULE. Nothing distinguishes a name-tier fact from a path-tier one from
   a content-tier one. The owner's examples straddle that line: hidden and gitignore are
   free during the walk; text/binary and real mime need BYTES, which is fdu's content tier
   -- separate and opt-in precisely because opening files is a different cost class.
   Without a declared tier, someone adds a `binary` tag and silently turns a metadata walk
   into a content walk.

3. THE ONE NAME THAT EXISTS IS THE HARDCODING. ignore_rules_fingerprint sits directly
   beside type_rules_fingerprint and reducers_fingerprint, both named for a CATEGORY of
   rule. Only this one is named for a specific policy. If gitignore is one tag among
   several it is tag_rules_fingerprint.

A DISTINCTION WORTH KEEPING: mime type is not a flag, it is categorical, and fdu already
has the generic mechanism for that -- ext_id/group_id, interned keys with maintained tally
maps. You would never want a plane per mime type. The honest model has two shapes and
neither should absorb the other: boolean tags (bitset -> optional plane) and categorical
keys (interned id -> tally map). Between them they cover the whole list.

ALSO FOLD IN LATER: Classification.flags (generated, vendored, documentation) are already
per-entry booleans, recomputed from the name on every query and never maintained. Those
are name-tier tag rules wearing a different hat.

PROPOSED, AWAITING THE OWNER'S CALL: rename the fingerprint field, decouple tags from
planes in the spec, add the tier concept, and re-scope this bead and fdu-7rwf so gitignore
reads as the first rule rather than the feature's name.
