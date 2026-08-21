---
type: is
id: is-01m0k512k9a6dq2k51fbfe5xn4
title: Machine formats are byte-compared but never parsed; yaml has never been validated at all
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels: []
dependencies: []
parent_id: is-01m0k4qrz1rb300efa1s5z86w6
created_at: 2026-08-21T21:53:27.657Z
updated_at: 2026-08-21T22:34:57.861Z
---
A byte-stable golden proves the output has not *changed*. It does not prove the output is
*valid*: a consistently malformed document passes forever, and the serializers here are
hand-written -- the project deliberately avoids serde -- so nothing else would catch it.

Audited 2026-08-21:

  json    parsed, by JSON.parse in scripts/content-selfcheck.mjs
  jsonl   never parsed. The watermark integration test greps the output with
          `listing.contains(...)`, which is a substring check
  yaml    never parsed anywhere, by anything

YAML is the sharp end. Its serializer is hand-written, its quoting rules are the fiddly
part of the format, and no YAML parser has ever read fdu's output.

Fix by consuming each format in a golden, which is what a session test is for -- run the
command, pipe it into a parser, print a field. That demonstrates the format working rather
than merely holding still, and the demonstration is the artifact a reader learns from.

  json   node -e "JSON.parse(...)"           -- already the pattern here
  jsonl  parse each line independently, which is the format's entire contract
  yaml   needs a parser: `yaml` as a pinned devDependency is the portable option, since
         CI installs neither jq nor yq and node ships no YAML support

The dependency question belongs to SUPPLY-CHAIN-SECURITY.md and the 14-day cool-off.

## Notes

CORRECTION: jsonl IS checked line by line (report_format tests, 'line is not a JSON document'). Two of three claims stand, one did not. Accurate picture: json is really parsed (JSON.parse in content-selfcheck.mjs); jsonl is only brace-balance checked by the hand-written is_valid_json, which would accept {"a": } -- structurally balanced, not valid; yaml has no check of any kind. The yaml gap is the real one and is unchanged.
