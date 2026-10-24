---
id: Q-1845
title: When an aircraft starts and stops being a tracked flying object, and when its sector is updated
status: answered
related: [S-0303, S-0309, S-0501, S-0209, S-0502]
answered-by: [S-0501]
---

## Need

S-0303 Terms count "an aircraft-kind object in ordinary flight" among the *tracked flying
objects*, and say that an object's sector is "updated at the movement events that its movement
spec defines". S-0309 claim 9 makes every automatic search that looks for aircraft scan those
tracked objects (the airborne scan), and a range search that looks for aircraft only scans
nothing else. So an anti-air defence finds an aircraft in flight only when the aircraft is
tracked.

The movement spec of aircraft, S-0501 (r40), defines flight, take-off and landing, but names no
event at which an aircraft starts or stops being tracked, and no event at which its sector is
updated. For jumpjets, S-0502 gives these events (claims 4, 13, 14 and 29); for aircraft no spec
does. Until M8 step A4 round 3 the engine never tracked an aircraft, so no anti-air-only
building ever took an aircraft in flight as its target on its own. Only damage from the
aircraft made it respond. That was the lead's play note 15 of 2026-10-24: anti-air defences
and units did not engage enemy aircraft, or did so too late.

The code is in `crates/deterministic/sim/src/air.rs` (`sync_tracking`, called from
`sync_listing`, and `leaving_map`).

## Questions

1. **Start.** When does an aircraft become a tracked flying object?
   - At the start of its take-off, or when it first leaves the ground (S-0501 Terms: "in
     flight")?
   - When it is placed on the map already in the air, for example on leaving a factory or
     arriving as a paradrop or reinforcement carrier?
   - When it leaves a landing pad or airfield dock?
2. **End.** When does it stop being tracked?
   - When landing starts, or at touchdown?
   - When it lands on a pad or dock, or goes inside a transport or building?
   - When it dies and falls, or only when it is removed?
3. **Landed aircraft.** Is a landed aircraft ever tracked? An example is a landable aircraft
   parked on open ground (S-0209 claim 13: listed in a cell layer).
4. **Sector updates.** Is the sector of a tracked aircraft updated:
   - at every movement update of its flight,
   - only when it crosses into another cell, or
   - at some other event?
   For S-0303 claim 11's ordering, which update moves it to the end of its new sector's list?
5. **Order.** When several aircraft start being tracked on the same tick, do they join the
   global order in update order, or in another order?
6. **Non-ordinary flight.** Are these "ordinary flight", and so tracked?
   - An aircraft hovering in place while it waits, attacks, or circles a target.
   - An aircraft under a temporary effect that holds it (S-0501 r40 mentions none).

## Possible observation

On a flat test map, place one anti-air-only defence. Let an enemy unarmed aircraft fly past it
on a move order and record:
- the first tick the defence turns towards it or fires;
- the same with the aircraft hovering, landing next to the defence, and parked on the ground;
- the same with the aircraft taking off from a pad inside the defence's range.

## Current workaround

An interim rule, cited as `Q-1845` in the code:

- An aircraft is tracked exactly while it is on the map and S-0501's in-flight test holds: it is
  not "on the ground", that is, its listing height is positive and so no cell lists it (S-0209
  claim 13).
- The test is applied after every movement update of the aircraft, on placement, on release
  from a dock, and on every cell crossing. The sector is updated at each of these, so an
  aircraft crossing into another sector joins the end of that sector's list (S-0303 claim 11).
- It stops being tracked when it is on the ground, when it goes inside or off the map, and when
  it is removed. S-0303 Terms already end a placement registration when the unit goes off the
  map.
- A dying aircraft that is still in the air stays tracked until it is removed.

With this rule, an anti-air-only defence acquires an enemy aircraft flying past on a move order
with no damage first, for human-owned and computer-owned houses alike
(`crates/deterministic/sim/tests/claims_s0309_airborne.rs`). The S-0303 flying candidates of
area damage also reach aircraft in flight now, as S-0303 states.
