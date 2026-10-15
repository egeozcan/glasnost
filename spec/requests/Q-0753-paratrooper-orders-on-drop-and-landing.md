---
id: Q-0753
title: A dropped paratrooper's behaviour and orders from drop to landing
status: open
related: [S-0522, S-0215, S-0214, S-0511]
answered-by: []
---

## Need

S-0522 claim 9 puts a dropped paratrooper on the map, falling. S-0511 claim 10 says a falling unit's behaviour processing still runs, and that a landing does not issue a guard order. Neither spec says which behaviour the paratrooper has when it appears. These points are open:

- Does it get the placement fall-back to rest (S-0215 Terms), as any placed object does, or something else?
- Does it get it at the drop or at the landing?

The answer decides what the paratrooper does on the ground: idle, guard, or acquire targets. The code is `transport::drop_out`, which goes through `objects::place`.

## Questions

1. Right after a successful drop, which behaviour does the paratrooper have? Does it have a target or a destination?
2. After it lands with no player order, which behaviour does it have? Does it acquire nearby enemies by itself?
3. Can a player give a falling paratrooper a move order? If so, is the order kept until it lands?

## Possible observation

Run S-0522 scenario P8 with an enemy unit placed within weapon range of the landing cell. Use the read-only trace to record each paratrooper's behaviour, target and destination. Record from the drop until 60 ticks after landing.

## Current workaround

- The paratrooper is placed with the ordinary placement, so it takes the placement fall-back to rest at the drop.
- After that, its behaviour follows the ordinary rules.

This choice is marked in the code with this request's ID.
