---
id: Q-1755
title: What an attacking aircraft does when its target goes off the map
status: answered
related: [S-0501, S-0215, S-0301, S-0507, S-0508]
answered-by: [S-0501]
---

## Need
S-0501 claim 32 says an aircraft in *attack* whose target is gone ends its run (claim 17). The
glossary keeps *off the map* apart from *removed*: a unit inside a transport or a building still
exists, so it is not gone. No approved spec says that an attacker loses a target that goes off the
map, apart from the override revert of S-0215 claim 10 and the passengers of S-0507. So the engine
keeps the target. In table T1 the firing check then gives an answer other than *ready* on every
attempt. With the re-approach flag off, the aircraft hovers at its firing spot in *firing* and
*refiring*, never fires and never goes home. With the flag on, it chooses new firing spots and
spends its rounds without firing, then goes home. The lead's play note 5 (M8 step A4: "Harriers that
lose their target before firing circle at random") may be this case. The engine needs the answer
for `crates/deterministic/sim/src/air.rs` (`behaviour_turn`, the target clear before processing)
and the T1 stages.

## Questions
1. An aircraft in *attack* flies toward an enemy vehicle or infantry unit. Before the aircraft
   fires, the target enters a transport (or a garrisonable building, a tank bunker, a refinery
   dock). Does the aircraft keep that object as its target? If not, on which tick does it lose
   it, and does its run then end as claim 17 says?
2. If it keeps the target, what does it do while the target is inside: hover at the firing
   spot, circle to new firing spots, spend its rounds without firing, or something else? Does that
   change with the re-approach flag?
3. If the target comes out again while the aircraft is still in *attack*, does the run go on
   against it?
4. Is the answer the same for a ground unit attacking that target?

## Possible observation
An armed, landable aircraft with two rounds and one free pad, ordered to attack an enemy infantry
unit 12 cells away. Two cells before the firing spot, order the infantry unit into a transport next
to it. Record the aircraft's target, behaviour, position, height and ammunition every tick for 600
ticks. Repeat with the re-approach flag on, then with the transport unloading the unit 100 ticks
later.

## Current workaround
The target is kept, as no spec clears it. The aircraft does what table T1 gives for the firing
check's answers. Only a target that no longer exists (removed) is cleared before processing, under
claim 32.
