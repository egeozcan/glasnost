---
id: Q-1261
title: When an entrant asks an absorbing building for contact, and when a waiting entrant retries
status: open
related: [S-0605, S-0213, S-0215]
answered-by: []
---

## Need
S-0605 claim 3 says the actor asks for contact "when a mobile actor with the enter behaviour gets
an absorbing building as its destination and holds no contact", and claim 4 says a waiting actor
asks again "in each tick". The engine needs the exact moment of both
(`crates/deterministic/sim/src/structures/absorb.rs`, `order_entered`, `waiting_turn`).

## Questions
1. Does claim 3 run when the enter order is applied (before or after the order's other effects),
   or at the actor's first *enter* processing?
2. When contact is not granted for a reason other than a contact already held (an empty link place
   but, say, an alliance refused in one direction, or health 0), does the actor wait in line or do
   something else?
3. In which part of the tick does the waiting actor's retry run (its own turn before or after its
   movement, or a building-side update)? Does a granted retry's *enter* start at once or only at
   the next ready moment?

## Possible observation
Two infantry ordered into a one-place absorbing building in the same tick; record per tick the
second unit's destination, behaviour and position from the order until it goes in.

## Current workaround
Claim 3 runs when the enter order is applied, after its ordinary effects. Contact not granted for
any reason starts waiting in line. The retry runs in the actor's own turn after its movement
(where the depot retry of S-0314 claim 23 runs), and the granted *enter* is queued without an
immediate start.
