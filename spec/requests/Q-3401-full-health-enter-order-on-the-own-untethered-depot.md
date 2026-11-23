---
id: Q-3401
title: A full-health untethered vehicle given an enter order on the depot it holds contact with
status: open
related: [S-0314]
answered-by: []
---

## Need

S-0314 r116 (claim 17) removes a driving vehicle's depot destination at its movement arrival. Before r116 a vehicle that had arrived at a depot without the pad flag still held the depot as its order-level destination, so our test of claim 20's full-health release (`claim_20_same_depot_enter_while_docked` in `crates/deterministic/sim/tests/claims_s0314_depot.rs`) could not tell whether the enter order gave it the depot as destination: before and after were the same. Since r116 they differ, and the test shows that our engine gives the destination in the untethered case. The engine must decide whether that is right.

Two readings of S-0314 meet here:

- Claim 20's second bullet: a vehicle in contact but not tethered is *approaching*, also when it is in *sleep* or working at a depot without the pad flag, and "given any other order loses its contact at once". The enter order on the same depot is such an order when the vehicle's behaviour is not *enter* (the first bullet ignores it only then). The order then runs on a depot that holds no contact, and claim 23's last paragraph says the vehicle "asks for entry at once. If it is refused, it takes no contact and does not queue." Claim 16 refuses a vehicle at full health. Nothing there removes the order-level destination that an ordinary enter order gives, so the vehicle holds the depot as destination with no contact. This is what our engine does.
- Claim 20's third bullet, for a *docked* (tethered) vehicle: "a docked vehicle at full health given this *enter* order is released by the order itself ... the order takes claim 23's full-health case, which gives it no destination and no queued entry ... So the tick end of the order's tick does not show that contact, whatever the depot's pad flag." The words "whatever the depot's pad flag" suggest that the same holds at a depot without the pad flag, where by the Terms before claim 20 a vehicle is never docked.

## Questions

1. A vehicle at full health, in contact with a depot without the pad flag (so untethered), standing at its docking point after its movement arrival (in *sleep*, or settling or working), is given an enter order on that same depot. After the order, does it hold the depot as its order-level destination, or none?
2. If it holds the depot as destination with no contact, does it then move (for example towards the docking point it already stands at), and does its *enter* behaviour become current and make further entry requests, or does the order end?
3. Does "whatever the depot's pad flag" in claim 20's third bullet mean that the untethered case takes claim 23's full-health case too, or does it only say that the contact is gone at the tick end in both cases?

## Possible observation

H18 a's full-health repeat at a depot without the pad flag: record the vehicle's order-level destination, contact and current and queued behaviours in the row of the order's tick and for some ticks after.

## Current workaround

Our engine keeps its current rule (claim 20's second bullet, then claim 23's empty-depot path: the destination is given, the request is refused, no contact, no queued entry). The test asserts the absence of a destination only in the tethered case, and for the untethered case asserts the lost contact and the absent queued entry, citing this request.
