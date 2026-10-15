---
id: Q-0696
title: What a person's stop order does to a hovering vehicle's movement
status: open
related: [S-0503, S-0213, S-0207]
answered-by: []
---

## Need

S-0213 claim 8 makes a stop order clear a unit's target, destination and queued destinations. For
a driving vehicle S-0207 claim 11 adds what happens to its motion (the target fraction is lowered
and the current track is finished). S-0503 claim 4 sets a hovering vehicle's ρ to 0 for "an
explicit immediate stop" and points to claims 7 and 12, which describe refusals, not orders. No
spec says what a stop order does to a hovering vehicle's heading point, throttle, hover speed and
ρ. A hover that keeps its heading point is still moving (Terms) and travels on to that point.

The code in `crates/deterministic/sim/src/hover.rs` (`stop_order`) and `orders.rs` needs it.

## Questions

1. Is a person's stop order an "explicit immediate stop" of S-0503 claim 4, which sets ρ to 0?
2. Does it clear the heading point (so the hover is no longer moving), or does the hover travel
   on to its heading point and then stop?
3. Does it zero throttle and hover speed, as a refused step does (claim 12), or leave them, as the
   switch-off stop request does (claim 10)?

## Possible observation

Order a hover at full speed across open water, then stop it on a chosen tick. Record its position
each tick until it rests, and whether it stops short of, at, or past the next cell centre.

## Current workaround

The stop order only does what S-0213 claim 8 says (destination, queued destinations and stored
path cleared). The hover keeps its heading point, throttle, hover speed and ρ, travels on to the
heading point and stops there, as a driving vehicle finishes its track. The choice is marked in
the code with this request's ID.
