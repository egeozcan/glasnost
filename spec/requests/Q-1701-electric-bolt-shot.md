---
id: Q-1701
title: The electric-bolt shot: when it hits, what it hits, and how it is drawn
status: answered
related: [S-0301, S-0514, S-0616, S-0102]
answered-by: [S-0316]
---

## Need

The glossary (*electric-bolt shot*) and S-0616 Open questions say that no approved spec owns the
electric-bolt shot itself. The shot has an end point, where S-0616 claim 1 makes one system of the
game-wide spark type. S-0514 claims 23 to 26 rank the electric-bolt flag among the line effects. The
seam `glasnost_sim::emitters::electric_bolt_sparks` is never called, and no weapon makes an electric
bolt (`docs/m7-claims/S-0616.md`, claim 1, electric-bolt row). A weapon with the flag set falls through
to the next line effect or to its projectile.

## Questions

1. **The shot.** Does a weapon with the electric-bolt flag still launch its projectile? If not, where
   and when does its warhead detonate: at the target's point or at the target cell, on the firing tick
   or later? Can it miss, and if it can, where is its end point?
2. **Range and line of fire.** Do the weapon's range and minimum range apply as for other weapons? Do
   walls, cliffs or buildings between the firer and the target stop or shorten the bolt?
3. **Burst and rate of fire.** With a burst above one, is each shot of the burst its own bolt with its
   own sparks?
4. **The sparks.** One spark system per bolt at its end point (S-0616 claim 1). Is that also so when
   the target dies from the shot, or when the shot is fired at the ground?
5. **Drawing.** How many lines make a bolt? What colours, and do they come from the weapon, the house
   or a fixed setting? How is the jagged shape made (number of bends, how far they stray), and does it
   change between image frames? How long does a bolt stay on screen, and does it follow a firer or
   target that moves meanwhile? (Its draw order among the beam effects is S-0102's and S-0514
   claim 26's.)
6. **Sound.** Is any sound played besides the weapon's report?

## Possible observation

A unit type with the electric-bolt flag firing at a vehicle, at a building, at the ground and at a target
behind a wall, at single shot and at burst 2. Record the damage times and the frames of the drawing.

## Current workaround

No weapon makes an electric-bolt shot, and the spark seam is never called. A weapon with the flag set
draws its next line effect, or none, and launches its projectile.
