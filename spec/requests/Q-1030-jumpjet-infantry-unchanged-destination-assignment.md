---
id: Q-1030
title: Whether a jumpjet-type infantry unit ignores an unchanged order-level destination assignment
status: open
related: [S-0502, S-0213, S-0107]
answered-by: []
---

## Need

S-0502 revision 6 claim 24 says that "an ordinary jumpjet vehicle ignores an unchanged order-level
destination, including empty-to-empty, unless its destination-refresh state requires reassignment",
and S-0213 claim 21 repeats this for vehicles. For jumpjet-type infantry the specs state only the
own-cell filter (S-0502 claim 23: a nonempty assignment whose cell equals the current cell is
ignored). Nothing says what an infantry unit does with a repeated nonempty assignment to the same
destination elsewhere, or with a repeated empty assignment. The code in `jump::take_destination` and
`jump::stop_ordered` has to decide: a repeated assignment either reroutes again (a search, and for
infantry the standing-position draw over 0 to 3) or is ignored.

## Questions

1. For jumpjet-type infantry that holds a movement commitment, is an assignment of the destination it
   already holds (the same cell or the same object) ignored, as for a vehicle, or does it search for
   a movement point again and take the standing-position draw?
2. For such infantry, is a repeated empty assignment (the order-level destination is already empty) an
   effective clear that reroutes, or ignored as a vehicle's is?
3. Does the answer to question 2 differ for a stop order and for a clear made by behaviour processing?

## Possible observation

S-0502 J22 with a stock jumpjet infantry unit: record the order-level destination, the movement point
and the random state around two successive stop orders, and around two successive move orders to
the same cell.

## Current workaround

Interim rule: infantry behaves as the specs say for a vehicle where they are silent. In
`jump::take_destination` an assignment equal to the destination the locomotor last took is ignored
(for infantry and vehicles alike, as before this revision); in `jump::stop_ordered` the suppression
of an empty-to-empty clear applies to vehicles only, so a stop order always reroutes a committed
infantry unit. Noted in `docs/development.md` (section *Simulation (M6c: jumpjets)*).
