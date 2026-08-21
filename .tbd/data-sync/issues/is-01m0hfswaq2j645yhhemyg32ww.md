---
type: is
id: is-01m0hfswaq2j645yhhemyg32ww
title: Re-verify or re-scope the README comparative speed claims before alpha
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-21T06:23:17.334Z
updated_at: 2026-08-21T06:23:17.334Z
---
The README's headline comparison names pdu, dust, and Go gdu on a 901,963-entry tree.
Two of those three cannot be re-verified on the current development machine: pdu is not
installed, and the `gdu` binary present is GNU coreutils du, not the Go gdu the claim
refers to.

A paired counterbalanced run on ~/.rustup (175,191 entries, 10 trials) on 2026-08-20 did
confirm the dust comparison directionally: fdu 0.220 s vs dust 0.299 s (+39.2%, 95% CI
+31.8% to +61.4%). Scalar class: fdu --view summary 0.168 s vs dumac 0.161 s (-2.2%, CI
-8.9% to +2.6%, a tie), diskus +44.0%, dua +64.9%, bsd-du +173.7%, gnu-du +249.9%.

Separately, the harness's own release-qualification table marks every comparator
`Confirmable: no` and rates fdu `inferior` to dust and ncdu on peak RSS (92.2 MiB vs 76.7
and 2.0), not on time. Before alpha, decide whether the README keeps comparative claims
naming tools the project cannot re-measure, and whether the RSS gap needs an answer or an
explicit note.
