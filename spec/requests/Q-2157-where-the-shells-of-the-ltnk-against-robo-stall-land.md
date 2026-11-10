---
id: Q-2157
title: Why do the hunters' shells in the LTNK against ROBO stall land outside every robot tank's damage reach?
status: answered
related: [S-0306, S-0301, S-0309, S-0405, S-0503]
answered-by: [S-0306]
---

## Need
M-0005 replays M-0001's twelve LTNK against ROBO games on stock data. Seven of them end at the
tick limit: after a last hunter hit early in the game, every hunter is in *hunt* with a disabled
robot tank as its target, no hunter changes cell, and the shelling hunters fire at the rate S-0301
Formula F1 gives, but every one of their shells lands farther than the warhead's reach from every
robot tank. Their shells come from groups that never detonate at the target's position. Some damage
a neutral bunker. M-0005 says what sets where a group's shells land is not recorded.

Our engine, built from the approved specs, has no stall: in every one of the twelve games the LTNK
group destroys every robot tank (`glasnost-headless arena --record M-0001`, the LTNK against ROBO
pairing: 12 row wins where the stock has 5 row wins and 7 games with no winner). S-0405 revision
47's claim 13 (a placed hovering vehicle disabled at its first heading-point choice) and claim 20
(a disabled vehicle is targeted, damaged and destroyed) are built and agree with M-0005; they do
not make our shells miss. No approved claim we found makes a shell aimed at a disabled hovering
vehicle that has sunk to the ground, or at its cell, land away from it.

The code concerned is the projectile flight of `projectile` (its aim point at launch and its
motion) and the hunters' target choice of `combat::behave` and `targeting`.

## Questions
1. In those games, where is a stall shell aimed at its launch (the target's position, its cell,
   or another point), and where does it detonate relative to that aim point?
2. Does a robot tank that was disabled while lifted, and so lowers to the ground (M-0005: height 0
   once disabled), keep a position or height that the hunters aim at while the shells hit the
   ground at another place? That is, does the aim use a height the target no longer has?
3. Does anything about the target (disabled, at height 0, of a hovering locomotor) change the
   projectile's flight, its inaccuracy or its detonation point, compared with an enabled ground
   vehicle at the same place?
4. What makes a hunter keep its fixed firing position for the rest of the game while its shells
   miss, instead of approaching (S-0309, S-0215 *hunt*)?

## Possible observation
M-0005 Procedure step 2 with the projectile's position read at every tick of its flight, for one
shelling hunter of a timeout game (A0, say), and the target's recorded position and height at its
launch tick and at its detonation tick.

## Current workaround
The approved rules, unchanged: our hunters hit and destroy every robot tank. The arena keeps the
pairing failing on its win test.
