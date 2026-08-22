---
type: is
id: is-01m0m3f9q8b0ywwn6sgypp34zw
title: Python binding hand-copies the 'full' view diagnostic and it has drifted
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-22T06:45:30.983Z
updated_at: 2026-08-22T06:45:30.983Z
---
The parity run caught two copies of one message that no longer agree.

crates/fdu/src/cli.rs:1351    invalid {flag} "full": it names the whole report and cannot be combined with another view
crates/fdu-py/src/lib.rs:970  invalid view "full": it names the whole report and cannot be combined

The binding's copy lost the trailing clause. Compare the analyze equivalent, which lives once in content_model.rs and reaches both surfaces intact -- that one has not drifted precisely because it was never copied.

Same root cause as fdu-ggux and the hardcoded contract() list: the binding keeps its own copies of things the library owns. The fix is to move the combination rule into the library beside ViewSpec::parse and have both surfaces call it, not to re-sync the two strings.
