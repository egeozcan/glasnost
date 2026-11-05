---
id: Q-2676
title: An aircraft that survives the hit it takes when a landing check finds no free cell
status: open
related: [S-0501, S-0302]
answered-by: []
---

## Need

S-0501 r134 claim 7: when a landing aircraft's check of its flight point's cell refuses that cell
and S-0212 claim 25's nearby search finds no free cell around the cell it is over, the aircraft
takes a hit of its current health with the demolition warhead, its flight point is cleared, and no
touchdown follows in that update. Its take-off has ended in that update. Open question 28 leaves
open what follows when it survives that hit. The engine meets the case only for a pad-bound
aircraft that is refused (claim 24), over a map with no free cell in reach, and with a hit that
leaves it alive (S-0302 claim 1: a type or a state that the demolition warhead does not destroy).

The engine also needs the take-off requests of that update: claim 7 says the take-off ends at the
update's take-off step, which makes claim 8's take-off requests for the height the aircraft has,
and claim 8's middle band has the body face the destination. With the flight point cleared, the
point the body faces is not given.

## Questions

1. After it survives, with its destination kept (a pad-bound aircraft in *enter* holds its pad as
   destination), does its next movement update hand that destination to its flight again, so it
   flies back toward the refused flight point and lands there again? Or does it hold where it is,
   or climb to cruise height and hover, until it gets a new destination?
2. With no destination (a flight point of its own, claim 6), does it hold its height and position,
   climb toward its cruise height, or come down where it is?
3. In the update of the hit, where its height lies above half its cruise height and at or below
   the upper threshold of claim 8, which facing does the body request, if any?
4. Does the surviving aircraft stay tracked (claim 47), and does any later movement update repeat
   the check and the hit?

## Possible observation

A pad-bound aircraft type with a strength that the demolition warhead's verses leave above the hit
(or an invulnerable state), sent to a pad whose docking position lies outside the foundation, on a
map where every cell around the foundation within reach holds a building or is otherwise refused
by the nearby search. Record each tick from the refusal to 300 ticks after it: position, height,
held height, destination, flight point, health, and whether a detonation above the terrain finds
it.

## Current workaround

As the code then does, cited `design: S-0501 Open question 28 and Q-2676`. In the hit's update the
take-off step makes claim 8's requests for the height the aircraft has with no point to face: the
aiming request above the upper threshold, and in the middle band only the request for full speed,
with no body request. The aircraft is then in flight, not landing, with its held height the
cruise height its take-off gave it, and it stays tracked. With its destination kept, its next
movement update hands that destination to its flight again as a new destination (claim 8), so it
flies back toward the refused flight point, starts to land there again and is refused, searched
for and hit again. With no destination it holds no flight point: it is not moving, so claim 6's
given-no-destination rule changes nothing for it. It climbs to its held height, and with no point
to slow toward its speed fraction approaches the target fraction it was left with: 0 from its
landing, so it stops over where it was hit, or 1 when the hit's take-off step requested full
speed, so it flies on along its heading.
