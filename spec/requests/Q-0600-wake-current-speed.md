---
id: Q-0600
title: Which speed makes a driving vehicle or a ship "moving now" for wakes?
status: open
related: [S-0207, S-0504, S-0203]
answered-by: []
---

## Need

S-0207 claim 23 (driving wakes) and S-0504 claim 2 (ship wakes) both define *moving now* as
"its body is turning, or it has a heading point and its current speed is above 0". Neither
spec defines "current speed". For a drive vehicle S-0207 has two candidates: the speed
fraction ρ (Terms), a double from 0 to 1, and the progress a moving tick earns,
*s* = trunc(*s*₂ × ρ) (claim 2), which is 0 while ρ is small but positive. Q-0590 asked the
same question for the harvesting sprite, and S-0402 claim 5a answered it for that sprite
alone (*s* > 0), so the answer cannot be borrowed here. The ship locomotor of S-0504 has its
own speed terms, so the ship rule may need its own answer.

The engine also needs the place of the wake in the tick: a wake's animation takes its
creation draws from the simulation stream (S-0109 claim 3), so where it is created in a
vehicle's turn decides the order of draws (S-0203).

Code: `glasnost_sim::drive::driving_wake` (crates/deterministic/sim/src/drive.rs).

## Questions

1. For a drive vehicle (S-0207 claim 23), is "current speed above 0" the speed fraction
   ρ > 0, or a positive progress *s* in that tick, or something else?
2. On a tick where the vehicle has a heading point with ρ > 0 but *s* = 0, does it create a
   wake (on a tick divisible by 10, on water, off a deck)?
3. For a ship (S-0504 claim 2), which quantity is the current speed?
4. Where in the vehicle's turn is the wake created: before or after that tick's movement
   update, and is the position the one before or after the move?
5. "Has a heading point": for a drive vehicle, is that an active track (S-0207), or a
   heading point in the sense of S-0212 Terms even between tracks?

## Possible observation

A drive vehicle that can enter water (amphibious), ordered from a standstill across water
cells and captured every tick: the first tick with a wake against the tick its position first
changes, with an accelerating type so that ticks of ρ > 0 and *s* = 0 occur; and the same
for a ship (scenario N3).

## Current workaround

A drive vehicle is moving now when its body is turning, or when it is on a track and its
latest drive tick earned *s* > 0 (`DriveState::step`, the measure S-0402 claim 5a uses).
The wake is created in the vehicle's turn right after its drive update, at the position
after the move. Noted as an interim rule in docs/development.md.
