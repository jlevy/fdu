---
type: is
id: is-01kzsa4vam9v8vk0e3034fm74n
title: "PR#6 C6: combining an unknown timestamp produces a falsely known timestamp"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.931Z
updated_at: 2026-08-11T21:02:38.931Z
---
crates/fdu/src/types.rs:225-241. combine(unknown, known) returns known, but zero means unknown. Parent would advertise an 'as of' time it cannot prove. Make unknown contagious. Medium.
