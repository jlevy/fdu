---
type: is
id: is-01m36bkfbxe11a31bv8s5zc9rr
title: Opened route accepts Delivery workers/order/accept_partial and silently ignores them
kind: bug
status: open
priority: 1
version: 2
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:19.868Z
updated_at: 2026-09-23T05:25:51.407Z
---
Stack review R115-2 (#115). execution.rs plan(.., Route::Opened) refuses cache, watch and content but accepts workers.scan, order and accept_partial; OpenedIndex::open copies only batch_size and into_parts hardwires threads=1 and BreadthFirst. The PR claims opened roots reject combinations they cannot honor. Fix: refuse non-default values with DeliveryUnsupported (or honor them) and extend route_delivery_matrix_rejects_contracts_the_route_cannot_execute.
