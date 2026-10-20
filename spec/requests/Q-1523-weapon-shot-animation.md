---
id: Q-1523
title: Which shot animation a weapon plays when it fires, and where
status: open
related: [S-0109, S-0301, S-0313, S-0609, S-0102, S-0202]
answered-by: []
---

## Need
S-0109 claim 29 lists "a weapon's shot animation in ordinary firing" as a creator of animations
(created during the firer's turn, after the shot's projectile enters play), and S-0301 claim 15
and S-0501 claim 20 mention the "muzzle animation" that some shots do not make. No spec says
which animation a shot makes. The weapon's rules section has an animation list, and nothing says
which entry is chosen, where the animation is placed, whether it follows the firer or which
draws its creation takes. Our simulation (`crates/deterministic/sim/src/combat/fire.rs`) makes
no shot animation, and the client draws a small yellow cross of its own at the fire point
instead. The lead's review of 2026-10-19 found the gattling stages not visible (a gattling's
stages fire different weapons, and their shot animations are what changes on screen) and a
red "explosion" on a gattling vehicle that is our placeholder.

## Questions
1. When a weapon with a non-empty animation list fires, which entry is played? Is it chosen by
   the firer's facing (or its turret's facing) among the list's entries, by a random draw, or is
   the first entry always used? If by facing, how are facings mapped to list positions for lists
   of 1, 8, 16 and other lengths?
2. If a draw is taken, from which stream (S-0202), and where does it fall relative to the shot's
   other draws?
3. Where is the animation created: at the fire point of S-0313 for the firing slot and burst
   (with its height), at the firer's position, or elsewhere? Is it attached to the firer so that
   it moves with it?
4. Which layer and palette does it take (S-0102 claim 3, S-0103 claim 8): is it an object's own
   animation (row 3) or an unowned one?
5. Which shots make no shot animation: void shots, drop attempts, garrisoned occupants (S-0313's
   muzzle points), passengers firing out of an open-topped transport, buildings?
6. Is anything else drawn at the muzzle for a weapon without an animation list?

## Possible observation
Give a vehicle with a turret a weapon whose animation list has eight distinct test animations
(named test art). Turn the turret through all eight facings and fire once per facing; record which
animation appears, where, and whether it follows the vehicle when it moves. Record the random
stream around the shot. Repeat with a list of one entry and of sixteen.

## Current workaround
No shot animation is made (docs/development.md, *Visual review 2026-10-19*), and the game client
no longer draws its own muzzle cross (the debug view still can). Gattling stage changes stay
invisible on screen apart from what S-0609 claim 14 gives the turret.
