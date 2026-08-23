---
type: is
id: is-01m0pqhv4ay2p7y9s8wcqrs0v2
title: "PR #38 review R9: a generator recipe without the generator revision is not a recipe"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:54.729Z
updated_at: 2026-08-23T07:34:39.361Z
closed_at: 2026-08-23T07:34:39.360Z
close_reason: "Fixed: exp-064's provenance pins generator blob 33a9e74 and names the hard-coded seed; the loop guide now requires a generator recipe to name the generator's revision. Verified the blob is unchanged at 703ceac, ac4806a and HEAD."
---
exp-064 tree_provenance names gen_tree.py with an entry count; the seed is hard-coded and any edit to the script silently changes the tree. A reconstructible generator recipe must pin the generator revision.
