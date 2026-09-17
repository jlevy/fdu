---
type: is
id: is-01m2phzn814exmf4ty5vw6zha0
title: "Content tier under the stored-state model: per-analyzer records, one definition per metric"
kind: epic
status: open
priority: 0
version: 8
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - content
  - design
  - release
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01kzyqkgbvy3cmmc3qx9zwfnzp
  - is-01kzyp8vpx1852y9sjnb7k6w2g
  - is-01m2pj058tjrqdmdexgt0r88q6
  - is-01m2pj0eknqard0k8za13xygp8
  - is-01m2phvqccpjdrbbsr5y3kcydr
created_at: 2026-09-17T02:08:59.648Z
updated_at: 2026-09-17T02:57:35.954Z
---
The design problem behind fdu-gija. The analysis request is index state, set when a tree is opened,
while every other report dimension (views, selection, size, ignored) is a Query parameter projected
over retained state. Reports therefore present whatever the content tier holds, and anything that lets
the tier hold more than was requested (containment reuse, a long-lived index, cache-only reads) leaks
cache history into answers. Coverage is one value per file for a whole analyzer set, so one analyzer's
Unsupported erases another's results, and "a wider record holds every metric a narrower request would
recompute" is not true of the data model.

Goal: carry the requested analyzer set on the report request; record coverage and metrics per analyzer
(fdu-ky5m, fdu-7dj6); project metrics, derived words and analysis metadata to the request; restore
containment reuse only on that basis; prove warm-equals-cold for every analyzer-set pair on every surface.
The spec needs a design revision before implementation (the content-metrics plan is in done/).
