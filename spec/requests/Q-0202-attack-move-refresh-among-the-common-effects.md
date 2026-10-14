---
id: Q-0202
title: Where the attack-move refresh falls among the other common effects of a fall-back to rest
status: open
related: [S-0215, S-0309]
answered-by: []
---

## Need

S-0309 claim 23 (b) moves the attack-move refresh into the common effects of a fall-back to
rest (S-0215 claim 13). The refresh can give the actor a destination and queue *move* with an
immediate start, or queue *attack* with a target. S-0215 claim 13's table has its own rows that
give a destination: the first approach destination, the next route point and the infantry guard
spot. That claim says which destination wins when several of *its* rows give one. It does not
place the attack-move refresh among those rows, or say which destination and queued behaviour
the actor is left with when the refresh resumes and another row also applies on the same
fall-back.

In the code, `process::fall_back` runs the refresh last among the common effects, after the
guard-spot row, so a resumed attack move's destination replaces a guard spot given in the same
fall-back.

## Questions

1. When an infantry unit with an attack-move memory that will be resumed (engaged, or in
   *guard*) falls back to rest while its guard spot is on another cell, which destination
   does it hold afterwards, and which behaviour is queued or current: the resumed attack move's
   or the guard spot's *move*?
2. The same question for a vehicle with approach destinations, and for an actor that follows a
   route.
3. When the refresh removes the memory (not engaged and not in *guard*), does a guard-spot or
   route row in the same fall-back still apply as it would without a memory?

## Possible observation

An infantry unit is given a guard spot on another cell, then an attack-move order. It is made to
engage an enemy, and then made to fall back while still engaged, for example because its target is
removed. Record its destination, current and queued behaviour on that tick.

## Current workaround

The refresh runs after the other common effects (`process::fall_back`). The interim row in
docs/development.md names this.
