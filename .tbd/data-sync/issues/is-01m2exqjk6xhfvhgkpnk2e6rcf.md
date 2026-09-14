---
type: is
id: is-01m2exqjk6xhfvhgkpnk2e6rcf
title: "Decide the Python name for the opened-root registry option: type_rules (current) or registry"
kind: task
status: closed
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T03:00:22.243Z
updated_at: 2026-09-14T14:46:28.472Z
closed_at: 2026-09-14T14:46:28.472Z
close_reason: "1566ded: kept the name type_rules; its OpenedOptions field doc now says it takes the File Rollup registry document (MetaBrowser's 'the registry') and is identified by type_rules_fingerprint"
resolution: null
duplicate_of: null
---
fdu-qlxz (commit 77e3afa on #48) added a field to the typed Python OpenedOptions that takes the registry document text and passes it to OpenOptions.types. The fixer named it type_rules, matching the type_rules_fingerprint identity the engine reports. The case against: the plan spec and the MetaBrowser notes call it the registry, so registry may read better to adapter authors. It is cheap to rename before the MetaBrowser adapter (fdu-2xfp) depends on it, and expensive after. Needs the user's call.

## Notes

2026-09-14 DECISION (user): keep the name type_rules, which matches the type_rules_fingerprint the opened root reports. Add a docstring saying it takes the File Rollup registry document, so an adapter author searching for 'registry' finds it.
