---
id: Q-0414
title: Orders to slave miners: harvest without a destination, stop and moves of a deployed one, and repeated harvest orders
status: answered
related: [S-0402, S-0213, S-0415]
answered-by: [S-0402]
---

## Need

S-0402 claim 22 says what a slave miner vehicle does with a harvest order
that gives a destination. Several neighbouring cases are not stated. The code
is `econ::slaves` (`process_harvest`, `vehicle_update`, `recall`) and
`orders` (the stop arm and behaviour orders, S-0213 claim 8 and table T3 h).

## Questions

1. A slave miner vehicle is given *harvest* with no destination (for example
   by a script, or when its factory releases it). What does it do: rest,
   seek a field near itself, or something else? After how many ticks does it
   next process?
2. Stop, and a move or attack order, given to a deployed slave miner (a
   building): are its slaves called back, as for the vehicle (S-0213 claim 8,
   T3 h)?
3. A harvest order with a destination is given while the vehicle is already
   driving to a site found for an earlier harvest order. Does it search for a
   site again near the new destination at once? While it drives, does it
   search again later in the same order?
4. A player packs up a deployed slave miner by hand. Does the vehicle then
   recall its slaves and seek a field at its first group update, as after an
   automatic pack-up (claim 22)?
5. A travelling slave miner reaches its site. Does its deployment attempt
   come at the next group update, or at once when it arrives?

## Possible observation

A purpose-made map with a slave miner vehicle of a human player and two
fields. Give it the orders above one at a time; record its behaviour,
destination, group state (whether slaves are out) and cell each tick.

## Current workaround

1. It queues *guard* and processes again after 1 tick.
2. Only a vehicle master recalls its slaves.
3. One site search per harvest order (per start of *harvest*); a new order
   searches again.
4. Yes, after any pack-up.
5. At the next group update.

Listed in docs/development.md (slave miners).
