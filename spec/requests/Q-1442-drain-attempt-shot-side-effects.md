---
id: Q-1442
title: What else a drain attempt does or skips compared with a shot
status: answered
related: [S-0608, S-0301, S-0312, S-0516, S-0501]
answered-by: [S-0608]
---

## Need
S-0608 claim 1 lists what a drain attempt does not do (projectile, damage, warhead effects,
ordinary-shot draws, rearm countdown, burst, ammunition). Other effects that the firing step
has around an ordinary shot are not named: whether the attempt counts as a shot for the "last
shot" record (used by S-0516 for uncloaking and the firing reveal), whether firing a draining
weapon reveals or uncloaks the firer, and which comes first when a weapon is both a suicide
weapon (S-0312) and a draining weapon. An aircraft drainer's spent rounds (S-0501) also follow
from whether the attempt counts as a shot made.

## Questions
1. Does a drain attempt reveal the firer to the target's house or uncloak a cloaked firer, as an
   ordinary shot does?
2. Does it count as the firer's most recent shot for the rules that look at the time since the
   last shot?
3. A weapon flagged both as a suicide weapon and as a draining weapon: does the shot kill the
   firer, attempt a drain, or both?
4. Does an aircraft with a draining weapon spend a round or a pass on a drain attempt?

## Possible observation
S-0608 scenario D1 part a with a cloaked or stealth variant of the Disc (named test data), and a
named variant whose draining weapon is also a suicide weapon.

## Current workaround
Interim, cited as Q-1442 in `crates/deterministic/sim/src/combat/fire.rs`: the drain test comes
after the suicide test (a suicide weapon kills its firer first); the attempt does not record a
last shot, does not reveal or uncloak the firer, and is not counted as a shot made, so an
aircraft spends no round on it.
