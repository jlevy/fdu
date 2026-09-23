---
type: is
id: is-01m36bkh85fc1z49e7fp2fcsz5
title: Write the unreleased-schema rule and restore CHANGELOG API-break and flag notes
kind: task
status: closed
priority: 1
version: 3
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:21.796Z
updated_at: 2026-09-23T08:07:01.494Z
closed_at: 2026-09-23T08:07:01.484Z
close_reason: "Fixed in f5ab69d1 and 397a0c3a on #117; delta-reviewed"
resolution: null
duplicate_of: null
---
Stack review R117-1..3 (#117). (1) docs/machine-output.md, CHANGELOG and the directory-query plan cite an accepted pre-1.0 schema policy that is not written; release-process.md says any machine-output field change bumps the schema. /7 is itself new in this unmerged stack (main emits /6), so state the rule where release-process.md states the bump rule: a schema version may change until the first release that emits it. (2) CHANGELOG lost the fdu-core render() -> Result<String> API break and the --format tree|paths|long, --tree, --long flag names that #103 recorded. (3) SKILL.md:260 (+ cli-surface golden) and design-principles.md:576 --modified-since examples lack --kind file. Nits: cli.rs --limit per-group wording for flat lists; docs/usage.md says long prints allocated size.
