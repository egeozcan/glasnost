---
id: Q-2801
title: The open cases of the orders reset at an owner change
status: answered
related: [S-0309, S-0215, S-0417, S-0408, S-0314, S-0406]
answered-by: [S-0309]
---

## Need

S-0309 r124 claim 31 gives the orders reset of every owner change that claim 30 lists. Our engine
builds it in one place, `structures::ownership::reset_orders`
(`crates/deterministic/sim/src/structures/ownership.rs`), called inside each listed change. The
claim's own Open questions leave four cases open, and three more come up in the build. Each one has
an interim in the code with a `// design: Q-2801` cite.

## Questions

1. (Claim 31's first Open question.) An object resumes an attack move at the change with only a
   remembered target, and the change has made that target an object of its own house or of an ally.
   Does *attack* fire on it, or drop it at its next check as an illegal target?
2. (Second Open question.) A building that is being sold when it changes owner gets no *guard*
   request, but the change's fall-back still requests its idle body. Does that request change the
   sale's body sequence, or the tick at which the sale ends?
3. (Third Open question.) A vehicle factory's unloading ends at the change because its *guard*
   starts at the request. What happens to a unit in contact that it has not yet sent out (still at
   its appear point), and what does the door do?
4. (Fourth Open question.) What does a vehicle docked at a captured repair building do after the
   change's guard request and fall-back? Does its repair go on?
5. A building still being built up (*construction* queued or current) changes owner, for instance
   by a trigger's change-owner action. Does the *guard* request replace its queued *construction*
   or end a current one? Does the fall-back's idle-body request take effect while it is built up?
6. The *rescue* rule says that the object "then has *guard* made current at once, without the
   readiness test". Does that hold for an object whose current behaviour is *wait* and whose queued
   behaviour is *rescue*? S-0215 claim 7 refuses its *guard* request, and S-0215 claim 9 refuses a
   forced *guard* while *wait* is current.
7. Psychic domination (S-0601 claim 13, S-0612 claims 13 and 14) changes a mobile object's owner
   after releasing it from any controller. Claim 30 drops its target ("whatever the cause"), but
   claim 30's list of causes and claim 31 do not name domination. Does a domination's owner change
   reset the object's orders as claim 31 gives?

## Possible observation

Questions 1 to 4 come with the scenarios that the claim's Open questions propose. For 5: a
trigger's change-owner action on a building placed by a place order, timed during its build-up,
recording its behaviours and body state each tick. For 6: a unit in *wait* with *rescue* queued,
given to another house by a trigger, recording its behaviours. For 7: a vehicle with a destination
and an attack-move memory, dominated, recording its behaviours and destination at the end of the
domination's tick.

## Current workaround

1. *Attack* treats the resumed target as any target it holds (S-0309 claims 25 part k and 29).
2. No idle-body request is made for a building whose current behaviour is *selling*, so the sale's
   body is not touched.
3. Nothing is added: the unit stays where it is, and the factory's next release goes on by S-0417's
   rules as they stand.
4. Nothing is added: the depot's rules (S-0314) go on as they stand with the new owner.
5. No idle-body request is made while *construction* is current or queued. The *guard* request is
   made as the claim gives (S-0215 claim 7's rows).
6. The rule forces *guard* (S-0215 claim 9), so with *wait* current the force is refused, *rescue*
   stays queued and *wait* stays current.
7. No reset: the domination drops the target (claim 30) and changes nothing else in the object's
   orders, as before claim 31.
