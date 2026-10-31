---
id: Q-2192
title: What an aircraft or building gets from a route-mode press inside a click area
status: open
related: [S-0213, S-0215]
answered-by: [S-0213]
---

## Need
S-0213 r74 claim 16d, *When the tests are made*: a press inside a click
area is not tested for kind. So in the original, a selected aircraft or
building with no route record joins a point as a vehicle does and gets a
route (low). A press inside a click area that is an ordinary click is not
refused for kind either. What either case then gives such an object has
not been studied (Open questions). Claim 16b also leaves open which orders
an ordinary click inside a click area gives an aircraft or building, and
whether any of them is not a behaviour order (as the detonate is for
vehicles and infantry).

Our engine sends the join. It needs a rule for what the route does
afterwards, and a rule for the ordinary click.

## Questions
1. An aircraft with no route record joins a point by a route-mode press.
   After the commit, does it issue the route's points? If it does, what
   does each point make it do: move, attack, or nothing? The same questions
   for a building, whose answer may be its own action.
2. Does such an object count toward the route-holder limit (claim 16)?
3. An aircraft or building is selected. A route-mode press lands inside a
   click area but is an ordinary click: for example, inside the click area
   of another house's point that is not drawn for the player, which the
   point-click test accepts but which is not the player's own point. What
   does the release give the object: a point add, a direct order, or
   nothing? Is any of these not a behaviour order?
4. When a point add for such an object is applied, is it refused? With
   which message, if any?

## Possible observation
Scenario O8 part x sets up an own building B and a vehicle route. Select an
own aircraft alone, hold the route-mode key, press on a point of the
vehicle's route and release the key. Record the aircraft's route record,
its orders and its movement for some seconds. Repeat with a building that
has a weapon, and with a factory. Then, with the aircraft selected, make a
press inside the click area of an enemy point that is not drawn for the
player, and record what the release sends.

## Current workaround
Our design, marked `// design (Q-2192)` in
`crates/deterministic/sim/src/routes.rs`:
- The join is sent and applied as it is for a vehicle, so the object gets
  a route and is a holder.
- When a point add for an aircraft or building is applied, it is dropped,
  with no message.
- At the release of an ordinary click, an order of any kind other than a
  behaviour order or a detonate is not sent.

What an aircraft or building that holds a route issues follows the
ordinary route-point issue (S-0215 claim 13), and it is not tested. With
`quirks.click_area_skips_kind_test` off, every press with such an object
selected is refused for kind, so none of this arises.
