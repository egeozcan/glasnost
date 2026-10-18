---
id: Q-1161
title: S-0215 claim 11's move-destination clause for a driving vehicle, and an ordered move from area guard, sleep or attack
status: answered
related: [S-0215, S-0206, S-0207, S-0213]
answered-by: [S-0215]
---

## Need

S-0215 revision 63, claim 11, now spells out "moving" for a vehicle with the driving locomotor.
It is moving while its body is still turning, "and also while the locomotor has a move
destination, or a next point whose map coordinates (x, y) differ from the vehicle's current
ones, height ignored (both clauses of S-0206's term ...)".

S-0207 claim 25 says the locomotor's move destination is set by an accepted movement
instruction before any path search succeeds, and that retained-goal waits, blockages and zero
speed do not clear it. S-0215 claim 12 says an order queues its behaviour without an immediate
start but sets its destination at once. So after a move order to a vehicle whose current
behaviour is *area guard*, *sleep* or *attack* with a target, the locomotor has a move
destination at the vehicle's next first per-tick start. That makes the vehicle moving and not
ready (none of the three exceptions applies), so the queued *move* would not become current
until the move destination is cleared, which normally happens at arrival. Meanwhile the old
behaviour would go on being processed. *Area guard*, for example, could send the vehicle back
towards its guard spot. The same holds for any vehicle whose move destination is kept while it
stands blocked or waits for a path retry.

Our engine has always judged "moving" for readiness by movement in progress (a track, a turn in
place before a step, or path steps left; `occupancy::in_progress`). An ordered move from these
behaviours therefore starts at the vehicle's first update after the order, as claim 12
describes for a ready actor. Implementing the move-destination clause literally would make
every such order wait. That is a much larger change than the turning clause that revision 63
announces, and it is not listed among its cross-spec changes.

## Questions

1. A driving vehicle whose current behaviour is *area guard* (or *sleep*, or *attack* with a
   target) is given a move order to a free cell a few cells away. At its first update after the
   order, does *move* become current, or does it stay queued while the vehicle drives? If it
   stays queued, when does it become current: at arrival, or earlier?
2. A driving vehicle stands blocked, or waits for a path retry, with its move destination kept
   (S-0207 claim 25) and a behaviour queued. Is it ready to change?
3. Is S-0215 claim 11's "move destination" the locomotor's own point of S-0207 claim 25, as
   S-0206's term says, at the moment of the per-tick start that follows an order?

## Possible observation

For question 1: an armed tracked vehicle resting in *area guard*, with a move order to a free
cell 5 cells away. Read the current and queued behaviour after every tick until it arrives.
For question 2: the same vehicle ordered into a pocket that a wall of other vehicles closes,
then given an attack order that keeps its destination, if one exists. Otherwise give it any
order that queues a behaviour while the destination stays.

## Current workaround

Interim rule, cited as "Q-1161 interim" in `docs/m7-claims/step1k-m7a.md` and
`docs/development.md` (Simulation, M7a: S-0215 revision 63). Apart from the turning clause,
"moving" for a vehicle's readiness is movement in progress as built (`occupancy::in_progress`:
a track, a turn in place before a step, or path steps left), not the presence of the
locomotor's move destination. An ordered move from *area guard*, *sleep* or *attack* starts at
the vehicle's first update after the order, and a blocked vehicle with no track and no path
steps is ready. `process::moving` adds only the turning clause of revision 63.
