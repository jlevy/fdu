---
type: is
id: is-01m2sm5dpddz0gsaxjf4hw9cen
title: "PR #86 Bugbot: raise README example count floor to 4"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m2sgadh8f84z9wxtrhhzszk9
created_at: 2026-09-18T06:44:48.966Z
updated_at: 2026-09-18T06:52:39.747Z
closed_at: 2026-09-18T06:52:39.734Z
close_reason: Raised the README example floor to 4 and required a compile-only watch fence in 9f1385f7. CI green, Bugbot thread resolved.
resolution: null
duplicate_of: null
---
Bugbot on 3c264ddb: comment says four Python blocks after splitting watch(), but assert ran >= 3. Dropping the watch fence still leaves the test green. Raise the floor to 4 and require at least one compile-only watch block.
