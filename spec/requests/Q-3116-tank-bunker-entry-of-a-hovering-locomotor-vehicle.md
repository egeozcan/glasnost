---
id: Q-3116
title: How a bunker-eligible vehicle with the hovering locomotor reports arrival at a tank bunker and is taken in
status: open
related: [S-0606, S-0503, S-0207]
answered-by: [S-0606]
---

## Need
S-0606 claim 1 excludes a vehicle from tank bunkers by its locomotion class only ("its locomotion
class is not *hover*"). The locomotor is a separate field (S-0503 claim 1: the locomotor field alone
chooses the model), so a turreted, armed type with the hovering locomotor and another locomotion
class (amphibious, for example) is bunker-eligible, is permitted (claim 2), takes the bunker's
contact and is sent to the docking point (claim 4).

Claim 5's arrival report "is made when the vehicle's movement brings it into a cell". The arrival
report then stops the vehicle, and claim 6's table S1 drives it in along the entry track and later
out along the release tracks. Every rule S-0606 gives for these steps is written for the driving
locomotor: the stop is S-0207 claim 11's, and the *driving in* stage and the releases use the driving
tracks (S-0207 tables T1 and T2). S-0503 has no bunker rows, and S-0606 has no hover rows.

In our engine a hovering vehicle reaches the docking cell and makes no report, so it stays on the
bunker's foundation in *enter*, holding the bunker's contact, and the bunker stays at *clearing* until
the vehicle is ordered away. (Issue #272.)

## Questions
1. Does a vehicle with the hovering locomotor make claim 5's arrival report when its movement brings
   it into the docking cell? At which point of S-0503's arrival handling (claim 15's table) is it
   made, before or after the path update and heading-point choice?
2. What does the arrival report's stop do to a hovering vehicle: S-0503 claim 10's stop request,
   claim 16's stop, or something else? Are its throttle, hover speed and heading point kept?
3. How does table S1's *driving in* stage move a hovering vehicle into the bunker, and how do the
   releases move it out: along the driving tracks with the hover height kept, by hover travel to a
   point, or by another rule? Does the entry change its height?
4. Is such a type perhaps refused earlier (claims 1 or 2) by a test the spec does not yet state?

## Possible observation
A test type with the hovering locomotor, amphibious locomotion class, a turret and a turret weapon,
ordered into an empty own tank bunker on flat ground. Record each tick from the arrival in the docking
cell: the bunker's stage, the vehicle's position, height and facing, and whether the vehicle is
taken in at all.

## Current workaround
None built: a hovering vehicle still makes no arrival report at a tank bunker, as before issue #272.
