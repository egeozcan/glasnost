---
id: Q-2971
title: Is there a bound on a route's length once joins have taken it past 16 points?
status: answered
related: [S-0213]
answered-by: [S-0213]
---

## Need

S-0213 claim 16b refuses a point add only when the route holds exactly 16 points, and lets a
press's join add its copy after the actor's own points "whatever their number"
(`quirks.press_copy_ignores_route_state`, default on). Read together, nothing in the spec stops a
route from growing: a route that a join has taken to 17 points takes point adds again (it no
longer holds exactly 16), and joins between two holders can double a route's length at each round.
GitHub issue #158 showed a single house, four orders per round, taking a route to millions of
steps in a few ticks.

Our engine now stops a route at 1024 steps as its own input validation (`routes::MAX_STEPS`,
marked `nocite`): a point add to a route of 1024 steps or more is refused with no message and no
sound, and a join whose copy would take the route past 1024 steps does nothing. 1024 is 64 times
the 16 points one route can be given by point adds alone; we believe no game a person plays comes
near it. We would like to know what the game does, in case a real route can pass it or the
refusal should look like one of claim 16b's.

## Questions

1. With the default press rules, can a route made by joins hold more than 16 points, and does a
   point add to a route of 17 or more points then add a point?
2. Is there any largest number of points (or of steps, counting a point that a route holds twice)
   that a route can reach? What happens to a point add or a join that would pass it: refused with
   the 16-point message, refused without one, or something else?
3. Does a join that would take a route past such a bound add part of its copy, or nothing?

## Possible observation

Two route-mode units X and Z of one house: give X a route of 16 points, give Z one point, then
join Z to X's first point (Z's route: 17 points). Add points to Z one by one and record whether
each is taken and which message shows. Then join X and Z to each other's last points in turns and
count the points each route draws after each round, until a join or add is refused.

## Current workaround

`routes::MAX_STEPS` = 1024 steps per route, `// nocite: our own input validation (#158)`, in
`routes::apply_point` and `routes::apply_join`, with tests in
`crates/deterministic/sim/tests/hostile_orders.rs`. Nothing changes for routes below that bound.
