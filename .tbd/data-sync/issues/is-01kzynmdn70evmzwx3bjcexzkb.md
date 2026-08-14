---
type: is
id: is-01kzynmdn70evmzwx3bjcexzkb
title: "Clarify and validate PR #15 content performance layers"
kind: task
status: in_progress
priority: 1
version: 26
labels: []
dependencies: []
child_order_hints:
  - is-01kzynyaz0kf8tzedtqddkw5zv
  - is-01kzyp8vpx1852y9sjnb7k6w2g
  - is-01kzyp9062ae6b5yy8nh3mtm0j
  - is-01kzypc08n60n25qy3508n129q
  - is-01kzypd1ywqq2g6evbtk3qk3cs
  - is-01kzypet6xhmxy6e5jtd17ww27
  - is-01kzypf1yd2v4g8q8tk2v1xmxs
  - is-01kzypyzsngc4w4209ec4abq9q
  - is-01kzypz62h7n4k9acw4s304fd5
  - is-01kzyqdc2hvymv0b082k9g65yc
  - is-01kzyqkgbvy3cmmc3qx9zwfnzp
  - is-01kzyqkmswz9qha4knttpt9xzw
  - is-01kzyqkxp6g2r367dvkxvvahec
  - is-01kzyr76mpg2emh1sf1zktb7y4
  - is-01kzysj77mzmh0rdct83a5rwaa
  - is-01kzyv83g6px1zbkrp5cj3bck2
  - is-01kzyx7d2nyh5gjx4gk6f4sfd0
  - is-01kzyzpjp5rz94mz5ma25t0tdz
created_at: 2026-08-13T22:59:35.718Z
updated_at: 2026-08-14T02:21:31.297Z
---

## Notes

PR #15 merged at 241a3c8 with the content-analysis hardening: eligible text is read through EOF with no size cap; expected binary/encoding/unsupported coverage is non-fatal; language summaries are aligned and metadata-only unless analysis is requested; help, README, design/spec/decision docs, benchmark contracts, and scratch-corpus cleanup guidance are aligned. Follow-up PR #18 at 4bce242 adds exact one-shot text walked/read/fresh/cache performance telemetry; machine formats and watch/lifecycle output omit it. Full local make check passes; PR #18 CI is in progress. Global fdu 0.0.1-dev+g4bce242 is installed. Remaining broader readiness gaps continue in the open child issues.
