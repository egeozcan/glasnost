---
id: Q-1180
title: Where in a unit's own update the berserk countdown runs, and whether it runs out of play or in a warp
status: answered
related: [S-0610, S-0203, S-0513, S-0215]
answered-by: [S-0610]
---

## Need
S-0610 claim 5 and F2 say a berserk unit's remaining time goes down by 1 "in each of its own
updates", and that expiry drops the target and queues guard or hunt. They do not say where in the
update this happens relative to the other parts of the same update: healing, behaviour
processing (which may make a queued behaviour current), target acquisition and firing. The
order is observable in the expiry update. If the countdown runs before behaviour processing, the
queued guard or hunt can become current, and a new target can be acquired, in the same update.
If it runs after firing, the unit can still fire once as berserk in its expiry update.

The engine also needs to know whether the countdown runs in updates that do only part of the
ordinary work. These are a unit out of play (a passenger; S-0610 open question 1) and a member
of an S-0513 warp whose update is the warp update alone (S-0203 claim 4).

The gap is in `crates/deterministic/sim/src/tick.rs` (the living part of `object_turn`, call to
`glasnost_sim::psychic::countdown_turn`).

## Questions
1. In the update where a berserk unit's remaining time reaches 0, can the unit still fire a shot
   in that same update before it stops being berserk?
2. In that update, does the queued guard (or hunt) already govern the unit, so that, for
   example, a guard-mode target search or a hunt move starts in the same update? Or does it
   start only in a later update?
3. Does the remaining time go down while the unit is a passenger inside a transport or
   building? Is it kept or cleared when the unit leaves?
4. Does the remaining time go down while the unit is being moved by a chronosphere-style warp
   (S-0513), in the updates where it takes no ordinary actions?

## Possible observation
Make a unit berserk with a known duration (S-0610 scenario P2) next to a target in range with a
rearm time that makes the fire update predictable. Record, update by update, the last shot fired
while berserk and the first behaviour change after expiry. For question 3, load the berserk unit
into a transport for longer than its duration and unload it. For question 4, warp a berserk
unit and compare its expiry update with an unwarped twin.

## Current workaround
Interim rule: the countdown runs at the start of the living part of the unit's own update. That
is after the expected-health and protection-tint bookkeeping and before sinking, self-healing,
behaviour processing, acquisition and firing. So an expiring unit does not fire as berserk in its
expiry update, and its queued behaviour may become current in that update. The countdown runs
only in updates of units in play. A warp member's warp-only update and a passenger take no
countdown step: the remaining time is kept unchanged until the unit takes ordinary updates
again.
