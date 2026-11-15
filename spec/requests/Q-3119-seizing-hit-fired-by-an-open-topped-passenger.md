---
id: Q-3119
title: Whether a seizing hit fired by a passenger of an open-topped transport seizes
status: answered
related: [S-0607, S-0507, S-0203]
answered-by: [S-0607]
---

## Need
S-0607 claim 2 says that a seizing hit does nothing at all when "the firer has left play". The
glossary keeps *in play* and *on the map* apart and gives a passenger of an open-topped transport
as one that is off the map and in play (S-0203 claim 3); S-0507 claim 14 lets such passengers
choose and fire weapons. S-0607 claim 21, on the other hand, says of the magnetic wave that
"leaving play includes going inside a transport, a bunker or a building", with no exception for
an open-topped transport. Read with claim 21's wording, an open-topped passenger has left play
and its seizing hit does nothing; read with the glossary, it is in play and the hit goes on to
the rest of claim 2 and to claims 3 to 5. Our engine tests the firer's being on the map
(`lift::seize_hit` in `crates/deterministic/sim/src/lift.rs`, issue #285).

If such a hit does seize, S-0607 claim 7 fixes the gripped unit's movement point from the
lifter's point, which for a mobile lifter is its heading point, or its own position when it
stands still. No claim says what that point is for a lifter that rides inside a transport, nor
whether the wave of claim 21 (which ends when its firer goes inside a transport) and the grip of
claim 12 (whose table has no row for the lifter boarding or riding a transport) behave as for a
lifter on the map.

## Questions
1. Does a seizing hit whose firer is a passenger of an open-topped transport (off the map, in the
   update order) do nothing at all, as claim 21's "leaving play" wording reads, or does it go on
   to end the firer's current grip and seize an eligible target?
2. If it seizes: what is the lifter's point of claim 7 for such a lifter (its own stored position,
   the transport's position, the transport's heading point)?
3. If it seizes: does the grip last while the lifter rides, and does it end when the transport
   moves, unloads the lifter or is destroyed? Does a magnetic wave form, given that claim 21 ends
   a wave whose firer is inside a transport?
4. Does a lifter that boards an open-topped transport while it grips a unit end that grip (claim
   12 has a row for a lifter that leaves play, "not applicable" for its target)?

## Possible observation
S-0607 scenario L9 with the firer given a seizing primary weapon and loaded into an open-topped
transport: record whether the target is seized, takes damage, and where its movement point lies,
with a magnetic wave or not.

## Current workaround
The hit does nothing at all when the firer is not on the map, which covers an open-topped
passenger (the reading of claim 21's "leaving play"). The passenger keeps firing its seizing
weapon with no effect.
