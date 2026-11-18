---
id: Q-3256
title: What draws a designation shot's line when its weapon has no laser flag
status: answered
related: [S-0519, S-0518, S-0514, S-0308]
answered-by: [S-0519]
---

## Need

An issue report says that in a skirmish on an imported install, an infantry designator ordered to attack an enemy building calls its air strike, but no beam is drawn from the designator to the building. The lead confirmed he means the designator's beam, not the target line of S-0301 claim 24.

The specs we have:

- **S-0519 claim 16.** A designation shot "is fired like any other shot (S-0301; S-0308 row 6 chooses the weapon)".
- **S-0518 claim 18, "Which shots draw a beam".** A shot draws a beam when the weapon that fires has the laser flag. For a vehicle, infantry unit or aircraft, the beam takes the weapon's laser duration, colours and house-colour flag, with thickness 2 when that flag is on and 1 otherwise.

We traced it on synthetic data, with an infantry designator whose S-0308 row 6 weapon has the air-strike warhead and the laser flag. The order fires that weapon. The shot reports its line, ending at the building's target point, for the drawing to turn into a beam (S-0518 claim 18). The strike is called.

A local check of an install's data (kept out of the repository) finds a case that the specs do not seem to cover:

- **If** a designator's S-0308 row 6 weapon has no laser flag, then by S-0518 claim 18 its shot draws no beam. This holds even when the same weapon gives a laser duration above 0, the house-colour flag and laser colour fields, and even when it has no electric-bolt or radiation-beam flag either.
- No other claim we have draws a line for a designation shot: S-0519 claims 16 to 26, S-0518 and S-0514 claim 23.

So under the specs as written, the line the lead expects is not drawn for such data.

## Questions

1. Does the original draw a line from a designator to its target for a designation shot whose weapon has no laser flag? If so, what decides that it is drawn: the air-strike warhead, the designator flag of the firer, the weapon's laser duration or house-colour flag, or something else?
2. If it is drawn, is it a beam with S-0518's parameters taken from the weapon (claim 18's row for a vehicle's, infantry unit's or aircraft's laser weapon)? Or does it have parameters of its own: colour, thickness, duration, fade and blink?
3. Is it created with each designation shot, or held for as long as the designator keeps its target, or for as long as the strike is in progress (S-0519 claims 17 to 19)? Is it drawn when the strike is not ready (claim 18, F5)?
4. Where are its end points: the designator's fire point (S-0518 claim 20) and the target's centre, its target point (S-0301), or the point the projectile reaches?

## Possible observation

Scenario LB2 or LB3 with an infantry designator whose S-0308 row 6 weapon has the air-strike warhead and no laser flag. Order it to attack an enemy building, once with its strike ready and once with its strike not ready. Record each tick:

- the shots it fires;
- any line or beam drawn from it, with its end points, colours, thickness and lifetime;
- the strike's state.

Repeat with the laser flag on, to compare.

## Current workaround

None. Our engine follows S-0518 claim 18 as written: a designation shot draws a beam only when its weapon has the laser flag. We change nothing until this is answered.
