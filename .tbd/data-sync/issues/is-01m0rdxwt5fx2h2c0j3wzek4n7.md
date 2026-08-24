---
type: is
id: is-01m0rdxwt5fx2h2c0j3wzek4n7
title: Vendor the File Rollup conformance packet and run it against fdu's classifier
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0rahh7entj80k486sxs5k45
hold: blocked
hold_until: null
created_at: 2026-08-23T23:05:12.772Z
updated_at: 2026-08-24T00:53:10.384Z
---
Second half of fdu-5q6e, split out because it needs an artifact that is not in this
repository. The first half -- the two extension levels themselves -- landed: logical_ext
derives up to two eligible trailing components per the format's rule, and
TypeRegistry::canonical_ext falls back to the trailing component for both rule lookup and
the roll-up bucket. The property the bead asked to pin holds and is now a test.

WHAT IS LEFT: vendor the File Rollup conformance packet at a reviewed metabrowser
revision, verify its manifest and hashes locally and in CI (no network fetch, no sibling
checkout), and run it against fdu's classifier as a third parity surface.

BLOCKED ON THE PACKET ITSELF. Today's packet carries matching-only cases, which pass
against a single level and hide exactly the gap this work closed -- it would have gone
green before the change and green after it. Before vendoring is worth doing, the packet
needs direct basename-to-logical-extension cases, including at least one name whose
logical extension is also its canonical one (archive.tar.gz), so a fixture can tell
correct fallback apart from never deriving a logical value at all.

Ask metabrowser for those cases first. Vendoring the packet as it stands buys a green
check that proves nothing.
