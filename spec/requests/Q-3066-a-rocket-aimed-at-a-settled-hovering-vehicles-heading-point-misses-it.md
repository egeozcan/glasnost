---
id: Q-3066
title: A rocket aimed at a settled hovering vehicle's heading point misses it
status: open
related: [S-0506, S-0503, S-0303, S-0405, M-0003, M-0016]
answered-by: []
---

## Need

S-0506 r41 (claim 14, *A hovering destination*, the answer to Q-3044) makes a hovering vehicle's
heading point *P* the aim point whenever the vehicle holds one, switched off or not. Its backing
says that the M-0003 row restated by M-0016 (the launchers win all 12 games against switched-off
hovering vehicles) is consistent with this, since the vehicle takes S-0303's area damage at its own
distance from the detonation point. Batch S8 built the answer (#191). In our engine the row stops
reproducing. ROBO against V3 goes from 0/12/0 (as in stock) to 0/0/12: every game runs to the tick
cap and no robot tank takes any damage.

A trace of one game (seed 0, ROBO on side A) shows why.

- **The robots hold *P*.** The front column of robot tanks takes the hunt's first destination
  and chooses *P* in the first tick, while not yet switched off. *P* is the centre of the
  neighbouring cell toward the launchers. The disabling of S-0405 claim 13 switches them off in
  that same tick. The stop request of S-0503 claim 10 clears the destination and keeps *P*. They
  settle at hover height 0 in their start cells and never move again.
- **The rockets aim at *P*.** Each rocket fixes its aim point at that *P*, one cell from the robot.
  By claim 19 it detonates at its nose point, which lies below the surface (the placement height
  paragraph: no correction to the terrain).
- **The detonation is out of reach.** The horizontal distance from the detonation to the robot is
  less than the warhead's spread radius. But S-0303 claim 7's spatial distance, with the depth
  below the surface included, is greater than the spread radius. So claim 3 leaves the robot out
  and no hit is made.
- **The position aim reached the robot narrowly.** With the aim at the robot's position (S5's
  interim), the nose point lands a short way from the robot. It is just as deep below the surface,
  and the robot is inside the spread radius by a small margin. That is the hit that won the
  launchers every game.

So, with claim 19's placement height, S-0303's distance, and *P* one cell from the settled vehicle,
claim 14's *P* and M-0003's outcome do not agree in our engine. One of the four differs from the
game, and I cannot tell which from the specs.

## Questions

1. In the stock games of M-0003's ROBO against V3 row (scenario R27 part f), does each robot tank
   hold a heading point when the launchers fire at it? If so, where is that point relative to the
   tank?
2. Where does each rocket detonate in those games, relative to the robot tank it was launched at
   (horizontal distance and height relative to the surface)? How much health does the tank lose
   from that detonation?
3. In the robot tanks' first tick, does the hunt's first destination, and with it the choice of a
   heading point (S-0503 claim 5), come before or after the disabling that switches them off
   (S-0405 claim 13)? A tank switched off before it takes that destination holds no *P*.
4. If they hold *P* and the rocket aims at it: is a rocket's nose point below the surface at that
   detonation, as claim 19's placement height allows? Is S-0303 claim 7's distance to a vehicle
   standing on the ground measured from that lowered point, including the depth?

## Possible observation

Scenario R27 part f, as S-0506 already states it: the M-0003 set-up replayed in stock with the
recorder. For each launch at a switched-off robot tank, record its *P* or none in the launch tick,
its position, the rocket's detonation point with its height, and the tank's health before and
after.

## Current workaround

The code follows S-0506 r41: `rocket::heading_point_with` gives *P* whenever the hovering vehicle
holds one. The oracle arena's M-0003 row ROBO against V3 fails its side outcome (3 of 35 scored
pairings failing, against 2 on base). No switch or interim was added. The trace is in the
git-ignored `local-lane/sim-issues/s8/` (`trace-robo-v3-*.txt`).
