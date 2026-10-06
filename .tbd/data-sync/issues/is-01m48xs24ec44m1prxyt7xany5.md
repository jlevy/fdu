---
type: is
id: is-01m48xs24ec44m1prxyt7xany5
title: "fdu 0.4.1: totals that match, and the 0.4.0 review follow-ups"
kind: epic
status: open
priority: 1
version: 8
labels: []
dependencies: []
child_order_hints:
  - is-01m482enn67fyky9w886zv0f44
  - is-01m4890ntmsbhjrc6accmtpay9
  - is-01m4890pbx6ptfke3z0np9wm57
  - is-01m4890qg7xpqq58xkk47gmx0j
  - is-01m4890r5ejj9emqn1vb9gptdn
  - is-01m489wty6g4m38wvps5k7snn5
  - is-01m482372az7g2jh21mv7abgm9
created_at: 2026-10-06T15:37:07.981Z
updated_at: 2026-10-06T15:37:14.759Z
---
Patch release after 0.4.0. Priority (maintainer, 2026-10-06): anything where totals do not match. P1: fdu-x3pq (documents TOTAL document_words/pages are pooled estimates that differ from the sum of the rows, in either direction; make logical words additive, e.g. the clamp per file, or otherwise make the total and the rows agree), with fdu-ao6i (test both directions). Related, larger: fdu-579b (hardlink attribution: totals that count a hardlinked file once per link differ from du; a design gate, may exceed a patch). Then the deferred review items from #177/#181: fdu-rdbq, fdu-0j83, fdu-r3k2, fdu-ki59, fdu-1iv2, fdu-jqfz, fdu-mvnp. Not here: fdu-ti4c (marking RequestError variants #[non_exhaustive] is breaking; next minor).
