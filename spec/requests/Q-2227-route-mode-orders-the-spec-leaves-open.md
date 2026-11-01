---
id: Q-2227
title: A building's other route-mode actions on an object, and a copied action a click would not give
status: answered
related: [S-0213, S-0214]
answered-by: [S-0213]
---

## Need
S-0213 r84 claim 16b leaves two route-mode outcomes open (Open questions):

- *Aircraft and buildings, Q-2192 question 3*: of a building's actions, the
  *attack* action gives a behaviour order and so a point add, and the
  rally-point and undeploy clicks on a cell send their orders directly.
  Which of a building's other actions on an object give behaviour orders,
  and which send their orders directly, has not been settled.
- *Joins*: a joining holder keeps the order recorded for the copied holder,
  so it can issue an action that a click would not give it, such as an
  engineer's *capture* copied to a tank. What such an order then does is not
  settled.

The engine must decide what to do with these clicks and these issued orders.

## Questions
1. In route mode, with a building selected, a click on an object gives the
   building one of these actions outside route mode: making a factory the
   primary factory, letting an absorbing building's occupants out, or
   setting a factory's rally point on an object. For each: does the click
   add a route point, send the order at once (as the rally point on a cell
   does), or do nothing?
2. A tank joins a point whose longest holder is an engineer whose order there
   is *capture* on an enemy building. When the tank issues that point, what
   does it do: try to capture, attack the building, move to it, or nothing?
   Does the tank's route go on to its next point afterwards?

## Possible observation
For 1: S-0213 Scenario O8 part h's third game, with a factory selected and
the press made on an own unit (rally on an object) or on the factory itself
(primary), recording whether a marker appears and whether the order takes
effect at once. For 2: an engineer routed to an enemy building, a tank joined
at that point, the commit, and a record of the tank's behaviour, target and
destination when it issues the point.

## Current workaround
1. `routes::click` sends none of these orders and adds no point (marked
   `design (Q-2227)` in `crates/deterministic/sim/src/routes.rs`).
2. The joining holder issues the recorded order as it is, and the ordinary
   order application decides what it does (S-0213 claim 5); no special rule
   is added.
