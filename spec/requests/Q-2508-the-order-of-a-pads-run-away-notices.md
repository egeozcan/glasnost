---
id: Q-2508
title: In which order the aircraft in contact with a pad receive its run-away notice
status: open
related: [S-0501, S-0314]
answered-by: []
---

## Need

S-0501 r110 claim 24 has a pad that starts selling send its run-away notice to every aircraft in
contact with it, approaching, in the docking stage or on the pad (S-0314 claim 22). Each notice
gives its aircraft a landing spot near its house's buildings (Formula F8) as its destination.
Where F8 takes draws, or where one aircraft's spot can change the spot found for the next, the
order of the notices changes the game. Neither spec gives that order.

## Questions

1. When a pad with several aircraft in contact starts selling, in which order do they receive
   the notice: by contact place, in the order their contacts were granted, or another order?
2. Does an aircraft's landing spot from the notice depend on the spots given to the aircraft
   notified before it in the same sale?

## Possible observation

A pad type with two contact places (named test copy), two aircraft in contact with it, one on the
pad and one approaching, the second granted contact first. Sell the pad and record the landing
spot each aircraft gets, and the scenario draws taken in that tick, for the two grant orders.

## Current workaround

`air::sale_started` notifies the aircraft in contact-place order (an aircraft with no recorded
place last), then by object key, as `depot::sale_started` does for vehicles. Cited `design:
S-0501 claim 24 … Q-2508`.
