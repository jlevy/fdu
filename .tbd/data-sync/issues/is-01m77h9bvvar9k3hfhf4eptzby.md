---
type: is
id: is-01m77h9bvvar9k3hfhf4eptzby
title: "Python: name the replacement for the 0.5.0 spellings SortKey.MTIME and Selection(modified_since=, modified_before=)"
kind: task
status: open
priority: 4
version: 1
labels: []
dependencies: []
parent_id: is-01m77c2ng8nt6bt6tr2xmjgbsh
created_at: 2026-11-12T09:26:41.787Z
updated_at: 2026-11-12T09:26:41.787Z
---
PR #211 review A2 gave SortKey('mtime') and Selection(sort='mtime') the engine's renamed-key sentence. SortKey.MTIME still raises AttributeError and Selection(modified_since=...)/modified_before= a TypeError naming no replacement, because Selection is a frozen slotted dataclass with a generated initializer. Decide whether a 0.5.0 Python caller should get a hint (e.g. an Enum __getattr__ hook, or a keyword check where Selection is built) or whether Python's own errors suffice.
