---
id: Q-2576
title: What being given no destination does to an aircraft whose landing is under way
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r120 claim 6 (*Landing where it is*, Q-2507) gives an aircraft in flight, moving and not
landing that is given no destination a new flight point with no destination object. Claim 8 sends
the landing case to Open question 28: what being given no destination does to an aircraft whose
landing is under way was not traced.

The engine meets this case often. A stop order can reach a landing aircraft, and so can every
rule that clears a destination: a pad's sale, a contact that ends, a behaviour that forgets its
destination. Claim 7's touchdown then compares the aircraft's position with its flight point's
cell, and claim 8's exception compares a new destination with that cell. So the engine must know
whether the landing keeps its flight point, takes a new one or holds none.

## Questions

1. An aircraft is landing (claim 6 has started its landing) with a destination and a flight
   point. It is given no destination (a stop order, say) before it touches down. Does its landing
   go on?
2. If it goes on, does the flight keep the flight point it is landing at? Or does it take a new
   one, as claim 6 gives one that is not landing? Or does it hold none?
3. At its touchdown, does claim 7 forget anything before the cell test? (With no destination and
   the flight point kept, the forget would turn on that flight point's cell.)
4. If the landing goes on, does claim 4's descent move still draw it toward the cell centre of the
   destination it had? (Claim 6 says the descent move does not apply with no destination.)

## Possible observation

AC4's same-cell part, with the stop order given in the cycle after the first tick at whose end the
aircraft's landing is under way above height 0. Record its position, height and speed fraction each
tick to touchdown, and whether it takes off. Then give it a cell destination in the cell where its
landing started, and record whether it takes off.

## Current workaround

`air::given_no_destination` lets the landing go on. The flight keeps the flight point it is landing
at, with no destination object (`AirState::own_point`), so claim 7's forget and claim 8's exception
compare with that point's cell. The descent move does not apply, as with no destination in claim
6. Cited `design: S-0501 Open question 28 and Q-2576`.
