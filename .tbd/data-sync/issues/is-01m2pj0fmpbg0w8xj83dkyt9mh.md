---
type: is
id: is-01m2pj0fmpbg0w8xj83dkyt9mh
title: Evaluate standard Rust YAML emitters against a reusable fdu YAML utility
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
dependencies:
  - type: blocks
    target: is-01m2pj0g6c5fswmdcbbzjhx0rx
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T02:09:26.675Z
updated_at: 2026-09-17T02:57:33.423Z
closed_at: 2026-09-17T02:11:49.369Z
close_reason: "Evaluated: no standard Rust YAML emitter is conformant for YAML 1.1 and 1.2 readers; recommend an owned policy and streaming JSON/YAML sink (see notes)"
resolution: null
duplicate_of: null
---
Candidates: saphyr, yaml-rust2, serde_yaml (archived), serde_yml, serde-saphyr, unsafe-libyaml /
libyaml-safer. Criteria: maintenance and cool-off, dependency and unsafe cost in fdu-core, serde
requirement, emitter conformance on an adversarial corpus under PyYAML (1.1), ruamel (1.2) and npm yaml
(1.1 and 1.2), output control and streaming for large reports. Evaluation started 2026-09-17.

## Notes

2026-09-17 RESULT. 121 adversarial strings emitted as {path: s} and loaded by PyYAML 6.0.3 (pure and C),
ruamel.yaml 0.19.1 (safe and rt), npm yaml 2.9.0 strict 1.2 and 1.1. Strings misread or rejected by at
least one parser: fdu 0.1.0 rules 21; fdu branch rules 4 (`._`, `._1`, `.1_0`, `.`; fuzz adds unescaped
U+2028/U+2029 beside a space); serde-saphyr 1.2.0 13; saphyr 0.0.12 18; serde_norway 25; ruamel dump 17;
frontmatter-format 19; PyYAML dump 14; npm stringify 32; prototype policy 0. No crate quotes for YAML 1.1
and 1.2 together, several emit C1/U+FFFE raw, and adopting one adds serde or a tree model plus
fast-moving releases against the 14-day cool-off (serde_yml is RUSTSEC-2025-0068; serde_yaml archived).
Recommendation: own zero-dependency policy plus one streaming sink for JSON and YAML (fdu-y4in); shared
corpus and parser matrix in CI (fdu-omo5). Evidence: attic/yaml-conformance-eval-2026-09-17 (local, gitignored).
