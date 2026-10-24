---
id: Q-1838
title: A visiting aircraft placed on the ground retreats at once and then ignores its orders
status: open
related: [S-0501, S-0506]
answered-by: []
---

## Need
S-0501 r40 rules, read together, make every aircraft that becomes visiting at its own placement
inside the usable area retreat before it has done anything, and then ignore every order:
- Terms and claim 1: placement height uses the visiting mark already present, and the type test
  (not both selectable and landable, or a camera weapon) sets the mark only afterward. So a type
  that is not landable, built at a pad or launched by a carrier inside the usable area, is placed
  on the ground and is visiting from then on.
- T2: on the ground, visiting, no passengers, not in a team: forgets target and destination;
  *retreat*. Its first fall-back to rest therefore makes *retreat* current (in `glasnost-sim`
  this comes in its first processing on the ground, before any order or launch pass reaches it).
- Claim 34: an aircraft in *retreat* that is not a strike aircraft ignores a request to queue any
  other behaviour entirely, with no start.

So such an aircraft ignores a player's *attack* and *move* orders, and a carrier slot aircraft of
that kind ignores T4's launch-pass *move* and *attack* (claim 35), and it flies off to its edge
cell. Scenario AC29 part g expects a type made not landable, built for the test house, to attack
the vehicle and end a run; part h expects the visiting slot aircraft of a carrier to attack and
end runs. Open question 22 asks only whether *retreat* persists into a slot aircraft's next
sortie, which suggests the first sortie is expected to happen. `glasnost-sim` follows the rules
as written, so its synthetic carrier test uses a landable slot type, and its lost-target test of
claim 45's visiting case marks a landable aircraft visiting by hand.

## Questions
1. Is a type that is not landable (or not selectable) placed on the ground inside the usable area
   when it has no visiting mark before placement, or does the type test decide the placement
   height for it too?
2. If it is placed on the ground: does its first fall-back to rest on the ground take T2's ground
   visiting row (*retreat*) before any order or launch pass, or does something (production
   release, a carrier's launch, a first order) come first?
3. Does claim 34's rule hold for such an aircraft's *retreat*, so that it ignores later *move* and
   *attack* requests, including a player's order and T4's launch-pass rows?
4. AC29 parts g and h: does the expected attack run of a type that is not landable (or not
   selectable) still stand, and from which placement height does it start?

## Possible observation
AC29 part g's set-up with the read-only recorder logging, from the tick the aircraft is placed,
its height, visiting mark, behaviour and queued behaviour each tick, and the player's attack order
given in the first tick it is accepted; and part h's set-up logging each slot aircraft the same
way from its launch placement through its first launch pass.

## Current workaround
The rules are applied as written: the aircraft is placed on the ground, marked visiting, its first
fall-back makes *retreat* current, and claim 34 drops later requests. The synthetic tests that
need an attacking slot aircraft or a visiting attacker use a landable type (the slot type of
`claims_s0504::carrier_and_missile_ship_fight`) or set the mark by hand
(`claims_s0501::lost_target_visiting_hunts_toward_the_edge_and_is_removed_at_the_border`). Both
cite Q-1838.
