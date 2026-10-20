---
id: Q-1580
title: How the target line of a vehicle with the target-laser flag is drawn
status: answered
related: [S-0301, S-0102, S-0103]
answered-by: [S-0301]
---

## Need
S-0301 claim 23 says that a vehicle whose type has the imported target-laser flag, owned by the
local player's house, shows a line towards its target for 15 ticks after each shot, as drawing
only. Its Open questions leave the drawing itself open: the line's two end points, its colour and
its thickness. The game client imports the flag (`TargetLaser`, `VehicleCombat::target_laser` in
`crates/deterministic/sim/src/combat/data.rs`) and could draw the line from the shot events
(`Effect::Shot`, `crates/app/src/mapview/live.rs`, `take_effects`), but cannot choose its look.

## Questions
1. Where does the line start: the shot's launch point, the vehicle's position, or another point
   of the vehicle (and at what height)? Does the start follow the vehicle during the 15 ticks?
2. Where does it end: the target's position, its aiming point, the target point of S-0306, or
   the point the projectile was aimed at? Does the end follow a moving target, and what happens
   when the target is removed or changes within the 15 ticks?
3. Its colour: a fixed colour, the house colour, or a colour from the rules (and which key)? Is
   it solid, dashed or faded over the 15 ticks?
4. Its thickness in screen pixels, and whether it is drawn over or under the objects (its place
   in S-0102's draw order).
5. Does a new shot within 15 ticks restart the 15 ticks, or add a second line?

## Possible observation
A vehicle type with the target-laser flag set and an ordinary weapon, owned by the local player,
firing at a stationary target and then at a moving one, with screen captures each tick of the
15 after a shot; the same with the target removed in the middle.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): the flag is imported, and the
game client draws no target line.
