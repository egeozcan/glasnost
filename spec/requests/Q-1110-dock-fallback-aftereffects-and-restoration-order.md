---
id: Q-1110
title: What "immobilized by teleport aftereffects" names, and when restoration inside a dock assignment takes the offered destination
status: open
related: [S-0207, S-0417, S-0502, S-0402]
answered-by: []
---

## Need

S-0207 revision 32 claim 27 (and S-0502 revision 8 claim 27) give the teleport-capable dock fallback the
predicate "not immobilized by teleport aftereffects". The spec defines no such state: Terms and
claim 20 name *phasing in* (claim 20 f) and the held state of a parasite attachment, but none of them is
called an aftereffect, and claim 27 says nothing else. The engine cannot place the predicate without a
state to read (`refresh::immobilized` in `crates/deterministic/sim/src/refresh.rs`).

The first row of claim 27's table restores a suspended predecessor in the assignment itself. The spec
does not say whether the restored (teleporting) locomotor then takes the same assignment's destination, or
whether the assignment is applied to the driving locomotor before the restoration.

S-0417 revision 19 claim 5 makes restoration depend on there being no driving movement report, and says
a retained locomotor destination point (the exit apron's centre) still prevents it after the exit track
is complete. After a factory exit no order-level destination exists, so S-0207 claim 25's track-completion
row does not clear the point. Our engine therefore keeps such a vehicle on its temporary driving
locomotor until a stop, a later move's arrival or another event removes the point.

## Questions

1. Which state is "immobilized by teleport aftereffects": phasing in after a jump, the held state, the
   time after a jump before the next one is allowed, or something else? Does it matter for a vehicle whose
   current locomotor is not teleporting?
2. When the first row of claim 27's table applies (restoration inside the assignment), does the restored
   teleporting locomotor take the destination offered by that very assignment (so the vehicle jumps), or is
   the destination installed for the driving locomotor first?
3. For a teleporting vehicle that left its factory and has no order-level destination: which event clears
   the retained exit destination point so that restoration can happen at a later fall-back to rest? Does a
   chrono miner, whose run-away reply queues harvest, drive its first leg to the ore with the driving
   locomotor before it is restored?

## Possible observation

S-0417 U6 and S-0207 D32 with the recorder of the movement report and the two pending states, on a
factory-produced chrono miner.

## Current workaround

Interim rules: the aftereffects predicate is a seam that answers no (`refresh::immobilized`); restoration
inside an assignment happens before the assignment's destination is taken, and judges the stored locomotor
destination point (not the offered one); the retained exit destination point prevents restoration as
S-0417 claim 5 says, with no other clearing event. Noted in `docs/development.md` (section *Simulation
(M6c: destination refresh and the dock fallback)*).
