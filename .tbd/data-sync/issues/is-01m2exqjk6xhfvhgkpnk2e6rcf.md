---
type: is
id: is-01m2exqjk6xhfvhgkpnk2e6rcf
title: "Decide the Python name for the opened-root registry option: type_rules (current) or registry"
kind: task
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T03:00:22.243Z
updated_at: 2026-09-14T03:00:22.243Z
---
fdu-qlxz (commit 77e3afa on #48) added a field to the typed Python OpenedOptions that takes the registry document text and passes it to OpenOptions.types. The fixer named it type_rules, matching the type_rules_fingerprint identity the engine reports. The case against: the plan spec and the MetaBrowser notes call it the registry, so registry may read better to adapter authors. It is cheap to rename before the MetaBrowser adapter (fdu-2xfp) depends on it, and expensive after. Needs the user's call.
