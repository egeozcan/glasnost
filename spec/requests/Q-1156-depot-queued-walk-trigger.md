---
id: Q-1156
title: When a queued depot vehicle makes the walk of S-0314 claim 23
status: answered
related: [S-0314, S-0215, S-0605]
answered-by: [S-0314]
---

## Need

S-0314 revision 24 claim 23 says a damaged vehicle given an enter order on a depot that holds a
contact gets a queued entry and, "at its first processing of *enter* (S-0215 claim 29) after
that", walks to a free location near the depot's centre cell. The same claim says the order gives
it no destination and no contact and that it "stays where it is, for now". Our engine applies
that order without changing the vehicle's behaviour (`depot::order_applied` ends the order), so
the vehicle never has *enter* current while its entry is queued, and no processing of *enter*
ever triggers the walk. S-0314's Open questions list both points: whether *enter* is current
during the wait, and whether the walk or the same turn's contact retry comes first (options A and
B).

## Questions

1. While a damaged vehicle waits with a queued depot entry, is *enter* its current behaviour (so
   that its behaviour processing makes the walk), or is its behaviour unchanged by the order?
2. If *enter* is not current, what starts the walk, and in which tick after the order?
3. In a turn where both are due, does the walk come before the contact retry (option B, a grant
   then reassigning the destination) or only when the retry finds no place (option A)?

## Possible observation

S-0314 scenario H18 f(ii) with the second vehicle's current behaviour and destination recorded
every tick from the order onwards.

## Current workaround

Cited as "Q-1156 interim" in `depot.rs`: in the vehicle's own queued-entry turn (after its
movement, where the retry runs), when the retry finds no free place and the vehicle holds no
contact, it makes the walk once per queued entry (option A): the nearby search of the Q-1152
interim from the depot's centre cell, the cell assigned as destination, *move* queued and started
at once, the queued entry kept. A new queued entry allows one new walk. This needs one new saved
flag per vehicle (`DepotState::walked`).
