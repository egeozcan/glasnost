---
id: Q-0087
title: Which orders count as a movement order for a prone unit's get-up
status: open
related: [S-0107, S-0213, S-0315]
answered-by: []
---

## Need
S-0107 claim 15 says that a prone infantry unit whose owner is
human-controlled asks (not forced) for *get-up* "when it is given a movement
order", unless its type is a fraidycat or a cyborg (S-0315 claim 3 refers to
the same rule). No approved spec defines *movement order*, and S-0213 gives
several orders that send a unit somewhere (`orders::prone_get_up`).

## Questions
1. Which orders make such a prone unit ask for *get-up*: a move order only,
   or also an attack-move order with a destination, an attack order on a
   target out of reach, an area-guard order, a queued move (waypoint) order,
   a scatter order, an enter or capture order?
2. Is the request made when the order is applied (S-0213 claim 3), or when
   the unit's locomotor starts moving?
3. Does a move order to the cell the unit already stands in count?

## Possible observation
S-0107 scenario M4, varied: a prone soldier of the human player given each of
the orders above in turn (each from a fresh prone state); record whether and
on which tick the get-up sequence starts.

## Current workaround
A behaviour order whose resulting behaviour is *move* (S-0213 claim 6,
attack-move translations included) counts; the request is made when the
order is applied. No other order makes the unit get up.
