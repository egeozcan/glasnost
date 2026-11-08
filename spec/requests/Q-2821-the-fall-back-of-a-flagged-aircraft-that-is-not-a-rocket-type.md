---
id: Q-2821
title: The fall-back of an aircraft of a missile-flagged type that is not a rocket type
status: open
related: [S-0501, S-0506]
answered-by: []
---

## Need

S-0501 r154 claim 18, *Missile types*, says that an object of a missile type (S-0506 Terms: a
type with the missile-spawn flag) takes T2's rows for an aircraft on the ground at every fall-back
that gets past T2's first three rows, whatever its height; T2's note says the same. S-0506 Terms
sets apart a *missile slot* (a slot whose spawned type is one of the three rocket types) from a
slot of a flagged type that is not a rocket type, which is a non-missile slot.

S-0501 claim 35 then says of a slot aircraft between an attack run's end and the next launch pass
that, not visiting, it "falls back to rest by T2's rows for its own type, as any armed aircraft in
flight does: the fall-back sets apart missile slot types (claim 18, *Missile types*), not other slot
aircraft". Read with S-0506's terms, "missile slot types" are the rocket types. So for the spawned
aircraft of a flagged type that is not a rocket type (S-0506 scenario R21's set-up), claim 18 gives
the rows for an aircraft on the ground (forget target and destination, *guard*), and claim 35 gives
the in-flight rows of an armed aircraft (its dock types and pad-bound field decide). The two
readings differ for every such aircraft in flight.

In our engine the rocket objects never reach T2's rows: S-0506's rocket flight drives them, and
`air::fall_back_rows` (`crates/deterministic/sim/src/air.rs`) leaves out every object that is not
an aircraft of our air movement. So the only objects that claim 18's *Missile types* reaches in our
engine are aircraft of a flagged type that is not a rocket type, the case where the two claims
differ.

## Questions

1. An aircraft of a type with the missile-spawn flag that is not one of the three rocket types,
   spawned as a slot aircraft or placed on its own, falls back to rest in flight past T2's first
   three rows. Does it take the rows for an aircraft on the ground (claim 18) or the in-flight rows
   of its own type (claim 35)?
2. Does the answer differ between such an aircraft spawned by a carrier and one that is not a
   slot aircraft (built at a pad, or placed by a map)?

## Possible observation

S-0506 scenario R21's named ordinary-flight release test map: its carrier spawns an ordinary
aircraft type with the missile-spawn flag, not a rocket type. Record, at the end of a released slot
aircraft's attack run in flight, its target, destination and current and queued behaviours over
the next ticks, before the next launch pass. The same type on its own (map-placed, armed, with dock
types and a pad on the map) after an attack run ends in flight.

## Current workaround

`// design: Q-2821` in `air::fall_back_rows`: the flag decides, as claim 18 and T2's note say. Such
an aircraft takes the rows for an aircraft on the ground at any height: it forgets its target and
destination and gets *guard* (or *area guard* for an armed aircraft of a computer-controlled
house); the after-the-row entries read its real height. Test:
`claim_18_r154_a_missile_type_takes_the_ground_rows_in_flight` in
`crates/deterministic/sim/tests/claims_s0501.rs`.
